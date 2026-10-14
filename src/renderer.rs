use anyhow::Result;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame as ImgFrame, RgbImage};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crate::config::ChartConfig;
use crate::interpolation::Frame;

pub struct Renderer {
    config: ChartConfig,
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

        for frame in frames {
            let mut buf = vec![0u8; (width * height * 3) as usize];

            {
                let root = BitMapBackend::with_buffer(&mut buf, (width, height)).into_drawing_area();
                root.fill(&WHITE)?;

                let mut chart = ChartBuilder::on(&root)
                    .caption(&self.config.title, ("sans-serif", 32).into_font())
                    .margin(30)
                    .x_label_area_size(40)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        0.0..(frame.max_value * 1.2).max(1.0),
                        0.0..self.config.top_n as f64,
                    )?;

                chart
                    .configure_mesh()
                    .disable_y_mesh()
                    .disable_y_axis()
                    .x_desc("Value")
                    .draw()?;

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

                    // Draw label: Name and value on/beside the bar
                    let label = if bar.value >= 1_000.0 {
                        format!("  {} ({:.0})", bar.name, bar.value)
                    } else if bar.value >= 10.0 {
                        format!("  {} ({:.1})", bar.name, bar.value)
                    } else {
                        format!("  {} ({:.2})", bar.name, bar.value)
                    };

                    chart.draw_series(std::iter::once(Text::new(
                        label,
                        (bar_len, y + 0.5),
                        ("sans-serif", 15).into_font().color(&BLACK),
                    )))?;
                }

                // Draw period/time label in bottom-right
                let period_color = BLACK.mix(0.65);
                let period_style = TextStyle::from(("sans-serif", 44).into_font())
                    .color(&period_color)
                    .pos(Pos::new(HPos::Right, VPos::Bottom));
                root.draw_text(
                    &frame.period_label,
                    &period_style,
                    (width as i32 - 40, height as i32 - 40),
                )?;

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
}
