use anyhow::Result;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame as ImgFrame, RgbImage};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crate::config::{ChartConfig, LabelPosition, NumberFormat, Orientation};
use crate::interpolation::Frame;

pub struct Renderer {
    config: ChartConfig,
}

fn to_superscript(exp: i32) -> String {
    let s = exp.to_string();
    s.chars()
        .map(|c| match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            _ => c,
        })
        .collect()
}

fn format_exp(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let abs_val = value.abs();
    let exp = abs_val.log10().floor() as i32;
    let mantissa = value / 10f64.powi(exp);

    if exp == 0 {
        return if mantissa.fract().abs() < 1e-4 {
            format!("{:.0}", mantissa)
        } else {
            format!("{:.1}", mantissa)
        };
    }

    let sup = to_superscript(exp);
    if (mantissa.abs() - 1.0).abs() < 1e-4 {
        if mantissa < 0.0 {
            format!("-10{}", sup)
        } else {
            format!("10{}", sup)
        }
    } else {
        format!("{:.1}×10{}", mantissa, sup)
    }
}

pub fn format_number(value: f64, format: NumberFormat) -> String {
    match format {
        NumberFormat::Auto => {
            if value >= 1_000.0 {
                format!("{:.0}", value)
            } else if value >= 10.0 {
                format!("{:.1}", value)
            } else {
                format!("{:.2}", value)
            }
        }
        NumberFormat::Integer => format!("{:.0}", value.round()),
        NumberFormat::Compact => {
            let abs_val = value.abs();
            if abs_val >= 1_000_000_000.0 {
                format!("{:.2}B", value / 1_000_000_000.0)
            } else if abs_val >= 1_000_000.0 {
                format!("{:.2}M", value / 1_000_000.0)
            } else if abs_val >= 1_000.0 {
                format!("{:.1}K", value / 1_000.0)
            } else if abs_val >= 10.0 {
                format!("{:.1}", value)
            } else {
                format!("{:.2}", value)
            }
        }
        NumberFormat::Scientific => format!("{:.2e}", value),
        NumberFormat::Exp => format_exp(value),
        NumberFormat::Fixed1 => format!("{:.1}", value),
        NumberFormat::Fixed2 => format!("{:.2}", value),
    }
}

impl Renderer {
    pub fn new(config: ChartConfig) -> Self {
        Self { config }
    }

    /// Render frames into an animated GIF using Plotters and the image crate
    pub fn render_gif<P: AsRef<Path>>(&self, frames: &[Frame], output_path: P) -> Result<()> {
        self.render_gif_with_progress(frames, output_path, |_, _| {})
    }

    /// Render frames into an animated GIF, calling `on_progress(current_frame, total_frames)` after each frame is processed
    pub fn render_gif_with_progress<P, F>(
        &self,
        frames: &[Frame],
        output_path: P,
        mut on_progress: F,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        F: FnMut(usize, usize),
    {
        let delay_ms = 1000 / self.config.fps.max(1);
        let file = File::create(output_path)?;
        let mut encoder = GifEncoder::new(file);
        encoder.set_repeat(Repeat::Infinite)?;

        let width = self.config.width;
        let height = self.config.height;

        let bar_label_size = self.config.effective_bar_label_size();
        let axis_label_size = self.config.effective_axis_label_size();
        let name_font = ("sans-serif", bar_label_size).into_font();
        let (margin_left, y_label_area_size) = match self.config.label_position {
            LabelPosition::YAxis => {
                let max_name_width = frames
                    .iter()
                    .flat_map(|f| &f.bars)
                    .filter(|b| b.rank < self.config.top_n as f64)
                    .map(|b| name_font.box_size(&b.name).map(|(w, _)| w).unwrap_or(0))
                    .max()
                    .unwrap_or(0);
                let y_desc_space = if self.config.y_label.is_some() {
                    axis_label_size * 2 + 6
                } else {
                    0
                };
                // Keep 8px padding between the category text and the Y-axis (x=0)
                let area_size = max_name_width + 8 + y_desc_space;
                (15, area_size)
            }
            LabelPosition::BarTip => {
                let area_size = if self.config.y_label.is_some() {
                    axis_label_size * 2 + 16
                } else {
                    15
                };
                (20, area_size)
            }
        };

        let total_frames = frames.len();
        for (i, frame) in frames.iter().enumerate() {
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
            on_progress(i + 1, total_frames);
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

        let bar_label_size = self.config.effective_bar_label_size();
        let axis_label_size = self.config.effective_axis_label_size();
        let period_label_size = self.config.effective_period_label_size();

        let x_label_area_size = if self.config.x_label.is_some() {
            axis_label_size * 2 + 10
        } else {
            axis_label_size + 8
        };

        let margin_top = if self.config.is_title_visible() { 6 } else { 8 };
        let margin_bottom = 8;
        let margin_right = 15;

        let mut builder = ChartBuilder::on(root);
        if self.config.is_title_visible() {
            let title_size = self.config.effective_title_size();
            builder.caption(&self.config.title, ("sans-serif", title_size).into_font());
        }
        builder
            .margin_top(margin_top)
            .margin_bottom(margin_bottom)
            .margin_left(margin_left)
            .margin_right(margin_right)
            .x_label_area_size(x_label_area_size)
            .y_label_area_size(y_label_area_size);

        macro_rules! draw_horizontal_chart {
            ($chart:ident, $base_val:expr) => {{
                let mut mesh = $chart.configure_mesh();
                mesh.disable_y_mesh();
                let axis_font = ("sans-serif", axis_label_size).into_font();
                mesh.x_label_style(axis_font.clone());
                mesh.y_label_style(axis_font.clone());
                mesh.axis_desc_style(axis_font);

                if let Some(ref y_label) = self.config.y_label {
                    mesh.y_desc(y_label);
                } else {
                    mesh.disable_y_axis();
                }

                if let Some(ref x_label) = self.config.x_label {
                    mesh.x_desc(x_label);
                }

                let axis_fmt = self.config.effective_axis_format();
                let bar_fmt = self.config.effective_bar_format();
                let formatter = move |v: &f64| format_number(*v, axis_fmt);
                if axis_fmt != NumberFormat::Auto {
                    mesh.x_label_formatter(&formatter);
                }

                mesh.draw()?;

                // Draw period/time label in bottom-right (clear of x-axis ticks)
                let period_style = TextStyle::from(("sans-serif", period_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Right, VPos::Bottom));
                let period_bottom_y =
                    height as i32 - (margin_bottom + x_label_area_size + 10) as i32;
                root.draw_text(
                    &frame.period_label,
                    &period_style,
                    (width as i32 - 40, period_bottom_y),
                )?;

                let (_, top_py) = $chart
                    .plotting_area()
                    .map_coordinate(&($base_val, self.config.top_n as f64));
                let (_, bottom_py) = $chart.plotting_area().map_coordinate(&($base_val, 0.0));

                let y_axis_name_style = TextStyle::from(("sans-serif", bar_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Right, VPos::Center));

                // Position value label vertically centered with the bar (VPos::Center)
                let val_text_style = TextStyle::from(("sans-serif", bar_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Left, VPos::Center));

                for bar in &frame.bars {
                    let y = self.config.top_n as f64 - 1.0 - bar.rank;

                    if y < -1.0 || y > self.config.top_n as f64 {
                        continue;
                    }

                    let color = Palette99::pick(bar.color_idx).filled();
                    let bar_len = bar.value.max($base_val);

                    // Draw horizontal bar rectangle from base_val to bar_len
                    $chart.draw_series(std::iter::once(Rectangle::new(
                        [($base_val, y + 0.15), (bar_len, y + 0.85)],
                        color,
                    )))?;

                    let val_str = format_number(bar.value, bar_fmt);

                    match self.config.label_position {
                        LabelPosition::YAxis => {
                            let (px, py) =
                                $chart.plotting_area().map_coordinate(&($base_val, y + 0.5));
                            if py >= top_py - 5 && py <= bottom_py + 5 {
                                root.draw_text(&bar.name, &y_axis_name_style, (px - 8, py))?;
                            }

                            if !self.config.hide_bar_labels {
                                $chart.draw_series(std::iter::once(Text::new(
                                    format!("  {}", val_str),
                                    (bar_len, y + 0.5),
                                    val_text_style.clone(),
                                )))?;
                            }
                        }
                        LabelPosition::BarTip => {
                            if !self.config.hide_bar_labels {
                                let label = format!("  {} ({})", bar.name, val_str);
                                $chart.draw_series(std::iter::once(Text::new(
                                    label,
                                    (bar_len, y + 0.5),
                                    val_text_style.clone(),
                                )))?;
                            }
                        }
                    }
                }
            }};
        }

        if self.config.log_scale {
            let log_min = self.config.log_min.max(1e-9);
            let max_val = max_val.max(log_min * 1.5);
            let mut chart = builder.build_cartesian_2d(
                (log_min..max_val).log_scale(),
                0.0..self.config.top_n as f64,
            )?;
            draw_horizontal_chart!(chart, log_min);
        } else {
            let mut chart =
                builder.build_cartesian_2d(0.0..max_val, 0.0..self.config.top_n as f64)?;
            draw_horizontal_chart!(chart, 0.0);
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

        let bar_label_size = self.config.effective_bar_label_size();
        let axis_label_size = self.config.effective_axis_label_size();
        let period_label_size = self.config.effective_period_label_size();

        let y_label_area_size = if self.config.y_label.is_some() {
            axis_label_size * 4 + 10
        } else {
            axis_label_size * 3 + 8
        };
        let name_space = match self.config.label_position {
            LabelPosition::YAxis => bar_label_size + 6,
            LabelPosition::BarTip => 8,
        };
        let x_desc_space = if self.config.x_label.is_some() {
            axis_label_size + 8
        } else {
            0
        };
        let x_label_area_size = name_space + x_desc_space + 8;

        let margin_bottom = 8;
        let margin_left = 15;
        let margin_right = 15;

        let mut builder = ChartBuilder::on(root);
        let (margin_top, period_top_y) = if self.config.is_title_visible() {
            let title_size = self.config.effective_title_size();
            builder.caption(&self.config.title, ("sans-serif", title_size).into_font());
            let m_top = 8;
            let p_y = (title_size as f32 * 1.1).round() as i32 + 8;
            (m_top, p_y)
        } else {
            let m_top = period_label_size + 10;
            let p_y = 6;
            (m_top, p_y)
        };
        builder
            .margin_top(margin_top)
            .margin_bottom(margin_bottom)
            .margin_left(margin_left)
            .margin_right(margin_right)
            .x_label_area_size(x_label_area_size)
            .y_label_area_size(y_label_area_size);

        macro_rules! draw_vertical_chart {
            ($chart:ident, $base_val:expr) => {{
                let mut mesh = $chart.configure_mesh();
                mesh.disable_x_mesh();
                mesh.disable_x_axis();
                let axis_font = ("sans-serif", axis_label_size).into_font();
                mesh.x_label_style(axis_font.clone());
                mesh.y_label_style(axis_font.clone());
                mesh.axis_desc_style(axis_font);

                if !self.config.log_scale {
                    mesh.y_labels(6);
                }
                if let Some(ref y_label) = self.config.y_label {
                    mesh.y_desc(y_label);
                }
                if let Some(ref x_label) = self.config.x_label {
                    mesh.x_desc(x_label);
                }

                let axis_fmt = self.config.effective_axis_format();
                let bar_fmt = self.config.effective_bar_format();
                let formatter = move |v: &f64| format_number(*v, axis_fmt);
                if axis_fmt != NumberFormat::Auto {
                    mesh.y_label_formatter(&formatter);
                }

                mesh.draw()?;

                // Draw period/time label in top-right for vertical orientation
                let period_style = TextStyle::from(("sans-serif", period_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Right, VPos::Top));
                root.draw_text(
                    &frame.period_label,
                    &period_style,
                    (width as i32 - 40, period_top_y),
                )?;

                let (left_px, _) = $chart.plotting_area().map_coordinate(&(0.0, $base_val));
                let (right_px, _) = $chart
                    .plotting_area()
                    .map_coordinate(&(self.config.top_n as f64, $base_val));
                let (_, top_py) = $chart.plotting_area().map_coordinate(&(0.0, max_val));
                let (_, bottom_py) = $chart.plotting_area().map_coordinate(&(0.0, $base_val));

                let name_style = TextStyle::from(("sans-serif", bar_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Center, VPos::Top));

                let val_style = TextStyle::from(("sans-serif", bar_label_size).into_font())
                    .color(&BLACK)
                    .pos(Pos::new(HPos::Center, VPos::Bottom));

                for bar in &frame.bars {
                    let x = bar.rank;

                    if x < -1.0 || x > self.config.top_n as f64 {
                        continue;
                    }

                    let color = Palette99::pick(bar.color_idx).filled();
                    let bar_height = bar.value.max($base_val);

                    $chart.draw_series(std::iter::once(Rectangle::new(
                        [(x + 0.15, $base_val), (x + 0.85, bar_height)],
                        color,
                    )))?;

                    let val_str = format_number(bar.value, bar_fmt);

                    let (px, py) = $chart
                        .plotting_area()
                        .map_coordinate(&(x + 0.5, bar_height));
                    let (base_px, base_py) =
                        $chart.plotting_area().map_coordinate(&(x + 0.5, $base_val));

                    match self.config.label_position {
                        LabelPosition::YAxis => {
                            if base_px >= left_px - 10 && base_px <= right_px + 10 {
                                root.draw_text(&bar.name, &name_style, (base_px, base_py + 6))?;
                            }
                            if !self.config.hide_bar_labels
                                && py >= top_py - 10
                                && py <= bottom_py + 10
                            {
                                root.draw_text(&val_str, &val_style, (px, py - 4))?;
                            }
                        }
                        LabelPosition::BarTip => {
                            if !self.config.hide_bar_labels
                                && py >= top_py - 10
                                && py <= bottom_py + 10
                            {
                                let label = format!("{} ({})", bar.name, val_str);
                                root.draw_text(&label, &val_style, (px, py - 4))?;
                            }
                        }
                    }
                }
            }};
        }

        if self.config.log_scale {
            let log_min = self.config.log_min.max(1e-9);
            let max_val = max_val.max(log_min * 1.5);
            let mut chart = builder.build_cartesian_2d(
                0.0..self.config.top_n as f64,
                (log_min..max_val).log_scale(),
            )?;
            draw_vertical_chart!(chart, log_min);
        } else {
            let mut chart =
                builder.build_cartesian_2d(0.0..self.config.top_n as f64, 0.0..max_val)?;
            draw_vertical_chart!(chart, 0.0);
        }

        Ok(())
    }
}
