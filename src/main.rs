use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use bar_chart_race::{
    ChartConfig, Interpolator, LabelPosition, Orientation, Renderer, TimeSeriesData,
};

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

    /// Label description for the X-axis (value axis)
    #[arg(long)]
    x_label: Option<String>,

    /// Label description for the Y-axis (category axis)
    #[arg(long)]
    y_label: Option<String>,

    /// Fixed upper limit for the X-axis
    #[arg(long)]
    max_x: Option<f64>,

    /// Clamp exit animations to bottom rank instead of actual rank
    #[arg(long)]
    clamp_exit: bool,

    /// Sort in ascending order (smaller values at the top)
    #[arg(long)]
    ascending: bool,

    /// Position of bar labels: 'y-axis' (beside Y axis) or 'bar-tip' (at the tip of each bar)
    #[arg(long, value_enum, default_value_t = LabelPosition::YAxis)]
    label_position: LabelPosition,

    /// Orientation of bars: 'horizontal' (default) or 'vertical'
    #[arg(long, value_enum, default_value_t = Orientation::Horizontal)]
    orientation: Orientation,
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
        x_label: args.x_label,
        y_label: args.y_label,
        max_x: args.max_x,
        clamp_exit: args.clamp_exit,
        ascending: args.ascending,
        label_position: args.label_position,
        orientation: args.orientation,
    };

    println!(
        "Interpolating frames (steps: {}, top: {}, order: {})...",
        args.steps,
        args.top_n,
        if config.ascending {
            "ascending"
        } else {
            "descending"
        }
    );
    let frames = Interpolator::generate_frames_from_config(&data, &config);
    println!("Total frames generated: {}", frames.len());

    println!("Rendering GIF to {:?}...", args.output);
    let renderer = Renderer::new(config);
    renderer.render_gif(&frames, &args.output)?;

    println!("Done! Saved to {:?}", args.output);
    Ok(())
}
