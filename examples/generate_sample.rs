use anyhow::Result;
use bar_chart_race::{ChartConfig, Interpolator, Renderer, TimeSeriesData};

fn main() -> Result<()> {
    let csv_path = "examples/data/cities.csv";
    println!("Reading {}...", csv_path);
    let data = TimeSeriesData::from_csv(csv_path)?;

    let config = ChartConfig {
        width: 800,
        height: 600,
        fps: 20,
        steps_per_period: 15,
        top_n: 5,
        title: "City Population Over Time (Millions)".to_string(),
        ..Default::default()
    };

    let frames = Interpolator::generate_frames(&data, config.steps_per_period, config.top_n);
    println!("Generated {} frames", frames.len());

    let output_path = "examples/output_cities.gif";
    let renderer = Renderer::new(config);
    renderer.render_gif(&frames, output_path)?;

    println!("Saved GIF to {}", output_path);
    Ok(())
}
