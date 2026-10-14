use anyhow::Result;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame as ImgFrame, RgbImage};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crate::config::{ChartConfig, LabelPosition, Orientation};
use crate::interpolation::Frame;

pub struct Renderer {
    config: ChartConfig,
}

fn format_value(value: f64) -> String {
    if value >= 1_000.0 {
        format!("{:.0}", value)
    } else if value >= 10.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    }
}

impl Renderer {
    pub fn new(config: ChartConfig) -> Self {
        Self { config }
    }

    /// Render frames into an animated GIF using Plotters and the image crate
    pub fn render_gif<P: AsRef<Path>>(&self, frames: &[Frame], output_path: P) -> Result<()> {
        let delay_ms = 1000 / self.config.fps.max(1);
        let file = File::create(output_path)?;
        let mut encoder = GifEncoder::new(file);
        encoder.set_repeat(Repeat::Infinite)?;

        let width = self.config.width;
        let height = self.config.height;

        let name_font = ("sans-serif", 15).into_font();
        let (margin_left, y_label_area_size) = match self.config.label_position {
            LabelPosition::YAxis => {
                let max_name_width = frames
                    .iter()
                    .flat_map(|f| &f.bars)
                    .filter(|b| b.rank < self.config.top_n as f64)
                    .map(|b| name_font.box_size(&b.name).map(|(w, _)| w).unwrap_or(0))
                    .max()
                    .unwrap_or(0);
                let y_desc_space = if self.config.y_label.is_some() { 30 } else { 0 };
                // Keep 8px padding between the category text and the Y-axis (x=0)
                let area_size = max_name_width + 8 + y_desc_space;
                (15, area_size)
            }
            LabelPosition::BarTip => {
                let area_size = if self.config.y_label.is_some() {
                    40
                } else {
                    15
                };
                (20, area_size)
            }
        };

        for frame in frames {
            let mut buf = vec![0u8; (width * height * 3) as usize];

            {
                let root =
                    BitMapBackend::with_buffer(&mut buf, (width, height)).into_drawing_area();
                root.fill(&WHITE)?;

                let max_val = self.config.max_x.unwrap_or(frame.max_value * 1.2).max(1.0);

                match self.config.orientation {
                    Orientation::Horizontal => {
                        self.render_horizontal_frame(
                            &root,
                            frame,
                            max_val,
                            margin_left,
                            y_label_area_size,
                        )?;
                    }
                    Orientation::Vertical => {
                        self.render_vertical_frame(&root, frame, max_val)?;
                    }
                }

                root.present()?;
            }

            let rgb_img = RgbImage::from_raw(width, height, buf)
                .ok_or_else(|| anyhow::anyhow!("Failed to create RgbImage from buffer"))?;
            let rgba_img = image::DynamicImage::ImageRgb8(rgb_img).into_rgba8();

            let img_frame = ImgFrame::from_parts(
                rgba_img,
                0,
                0,
                Delay::from_saturating_duration(Duration::from_millis(delay_ms as u64)),
            );
            encoder.encode_frame(img_frame)?;
        }

        Ok(())
    }

    fn render_horizontal_frame(
        &self,
        root: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        frame: &Frame,
        max_val: f64,
        margin_left: u32,
        y_label_area_size: u32,
    ) -> Result<()> {
        let (width, height) = root.dim_in_pixel();

        let mut chart = ChartBuilder::on(root)
            .caption(&self.config.title, ("sans-serif", 32).into_font())
            .margin(25)
            .margin_left(margin_left)
            .x_label_area_size(40)
            .y_label_area_size(y_label_area_size)
            .build_cartesian_2d(0.0..max_val, 0.0..self.config.top_n as f64)?;

        let mut mesh = chart.configure_mesh();
        mesh.disable_y_mesh();
        if let Some(ref y_label) = self.config.y_label {
            mesh.y_desc(y_label);
        } else {
            mesh.disable_y_axis();
        }

        if let Some(ref x_label) = self.config.x_label {
            mesh.x_desc(x_label);
        }
        mesh.draw()?;

        // Draw period/time label in bottom-right (clear of x-axis ticks)
        let period_style = TextStyle::from(("sans-serif", 52).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Right, VPos::Bottom));
        root.draw_text(
            &frame.period_label,
            &period_style,
            (width as i32 - 50, height as i32 - 65),
        )?;

        let (_, top_py) = chart
            .plotting_area()
            .map_coordinate(&(0.0, self.config.top_n as f64));
        let (_, bottom_py) = chart.plotting_area().map_coordinate(&(0.0, 0.0));

        let y_axis_name_style = TextStyle::from(("sans-serif", 15).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Right, VPos::Center));

        // Render bars smoothly based on continuous rank
        for bar in &frame.bars {
            // Continuous Y position: rank 0 -> top_n - 1.0, rank (top_n - 1) -> 0.0
            let y = self.config.top_n as f64 - 1.0 - bar.rank;

            // Skip if completely out of visible range
            if y < -1.0 || y > self.config.top_n as f64 {
                continue;
            }

            let color = Palette99::pick(bar.color_idx).filled();
            let bar_len = bar.value.max(0.0);

            // Draw horizontal bar rectangle
            chart.draw_series(std::iter::once(Rectangle::new(
                [(0.0, y + 0.15), (bar_len, y + 0.85)],
                color,
            )))?;

            let val_str = format_value(bar.value);

            match self.config.label_position {
                LabelPosition::YAxis => {
                    let (px, py) = chart.plotting_area().map_coordinate(&(0.0, y + 0.5));
                    // Only draw category name beside Y-axis if within visible vertical plot bounds
                    if py >= top_py - 5 && py <= bottom_py + 5 {
                        root.draw_text(&bar.name, &y_axis_name_style, (px - 8, py))?;
                    }

                    chart.draw_series(std::iter::once(Text::new(
                        format!("  {}", val_str),
                        (bar_len, y + 0.5),
                        ("sans-serif", 15).into_font().color(&BLACK),
                    )))?;
                }
                LabelPosition::BarTip => {
                    let label = format!("  {} ({})", bar.name, val_str);
                    chart.draw_series(std::iter::once(Text::new(
                        label,
                        (bar_len, y + 0.5),
                        ("sans-serif", 15).into_font().color(&BLACK),
                    )))?;
                }
            }
        }

        Ok(())
    }

    fn render_vertical_frame(
        &self,
        root: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        frame: &Frame,
        max_val: f64,
    ) -> Result<()> {
        let (width, _) = root.dim_in_pixel();

        let y_label_area_size = if self.config.y_label.is_some() {
            65
        } else {
            50
        };
        let x_label_area_size = if self.config.x_label.is_some() {
            60
        } else {
            40
        };

        let mut chart = ChartBuilder::on(root)
            .caption(&self.config.title, ("sans-serif", 32).into_font())
            .margin(25)
            .margin_top(40)
            .x_label_area_size(x_label_area_size)
            .y_label_area_size(y_label_area_size)
            .build_cartesian_2d(0.0..self.config.top_n as f64, 0.0..max_val)?;

        let mut mesh = chart.configure_mesh();
        mesh.disable_x_mesh();
        mesh.disable_x_axis();
        mesh.y_labels(6);
        if let Some(ref y_label) = self.config.y_label {
            mesh.y_desc(y_label);
        }
        if let Some(ref x_label) = self.config.x_label {
            mesh.x_desc(x_label);
        }
        mesh.draw()?;

        // Draw period/time label in top-right for vertical orientation
        let period_style = TextStyle::from(("sans-serif", 52).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Right, VPos::Top));
        root.draw_text(&frame.period_label, &period_style, (width as i32 - 40, 45))?;

        let (left_px, _) = chart.plotting_area().map_coordinate(&(0.0, 0.0));
        let (right_px, _) = chart
            .plotting_area()
            .map_coordinate(&(self.config.top_n as f64, 0.0));
        let (_, top_py) = chart.plotting_area().map_coordinate(&(0.0, max_val));
        let (_, bottom_py) = chart.plotting_area().map_coordinate(&(0.0, 0.0));

        let name_style = TextStyle::from(("sans-serif", 13).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Center, VPos::Top));

        let val_style = TextStyle::from(("sans-serif", 13).into_font())
            .color(&BLACK)
            .pos(Pos::new(HPos::Center, VPos::Bottom));

        for bar in &frame.bars {
            let x = bar.rank;

            // Skip if completely out of visible range
            if x < -1.0 || x > self.config.top_n as f64 {
                continue;
            }

            let color = Palette99::pick(bar.color_idx).filled();
            let bar_height = bar.value.max(0.0);

            // Draw vertical bar rectangle
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x + 0.15, 0.0), (x + 0.85, bar_height)],
                color,
            )))?;

            let val_str = format_value(bar.value);

            let (px, py) = chart.plotting_area().map_coordinate(&(x + 0.5, bar_height));
            let (base_px, base_py) = chart.plotting_area().map_coordinate(&(x + 0.5, 0.0));

            match self.config.label_position {
                LabelPosition::YAxis => {
                    // Category name below X-axis (base of bar)
                    if base_px >= left_px - 10 && base_px <= right_px + 10 {
                        root.draw_text(&bar.name, &name_style, (base_px, base_py + 6))?;
                    }
                    // Value above bar tip
                    if py >= top_py - 10 && py <= bottom_py + 10 {
                        root.draw_text(&val_str, &val_style, (px, py - 4))?;
                    }
                }
                LabelPosition::BarTip => {
                    // Name and value together above bar tip
                    if py >= top_py - 10 && py <= bottom_py + 10 {
                        let label = format!("{} ({})", bar.name, val_str);
                        root.draw_text(&label, &val_style, (px, py - 4))?;
                    }
                }
            }
        }

        Ok(())
    }
}
