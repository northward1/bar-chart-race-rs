use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use bar_chart_race::{ChartConfig, Interpolator, Renderer, TimeSeriesData};

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate Bar Chart Race animations in Rust", long_about = None)]
struct Args {
    /// Path to input CSV file (wide format)
    #[arg(short, long)]
    input: PathBuf,

    /// Path to output GIF file
    #[arg(short, long, default_value = "output.gif")]
    output: PathBuf,

    /// Chart title
    #[arg(short, long, default_value = "Bar Chart Race")]
    title: String,

    /// Number of top bars to display
    #[arg(short = 'n', long, default_value_t = 10)]
    top_n: usize,

    /// Frames per second
    #[arg(long, default_value_t = 30)]
    fps: u32,

    /// Intermediate steps between periods
    #[arg(short = 's', long, default_value_t = 15)]
    steps: usize,

    /// Width of the output image
    #[arg(long, default_value_t = 1024)]
    width: u32,

    /// Height of the output image
    #[arg(long, default_value_t = 768)]
    height: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Loading data from {:?}...", args.input);
    let data = TimeSeriesData::from_csv(&args.input)?;

    let config = ChartConfig {
        width: args.width,
        height: args.height,
        fps: args.fps,
        steps_per_period: args.steps,
        top_n: args.top_n,
        title: args.title,
    };

    println!(
        "Interpolating frames (steps: {}, top: {})...",
        args.steps, args.top_n
    );
    let frames = Interpolator::generate_frames(&data, config.steps_per_period, config.top_n);
    println!("Total frames generated: {}", frames.len());

    println!("Rendering GIF to {:?}...", args.output);
    let renderer = Renderer::new(config);
    renderer.render_gif(&frames, &args.output)?;

    println!("Done! Saved to {:?}", args.output);
    Ok(())
}
