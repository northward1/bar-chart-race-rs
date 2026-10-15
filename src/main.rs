use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use bar_chart_race::{
    ChartConfig, Interpolator, LabelPosition, NumberFormat, Orientation, Renderer, TimeSeriesData,
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

    /// Use logarithmic scale for values
    #[arg(long)]
    log_scale: bool,

    /// Baseline / minimum positive value for logarithmic scale
    #[arg(long, default_value_t = 1.0)]
    log_min: f64,

    /// Number format for values and axis ticks: 'auto', 'integer', 'compact', 'scientific', 'exp', 'fixed1', 'fixed2'
    #[arg(long, value_enum, default_value_t = NumberFormat::Auto)]
    number_format: NumberFormat,

    /// Override number format specifically for axis ticks
    #[arg(long, value_enum)]
    axis_number_format: Option<NumberFormat>,

    /// Override number format specifically for bar value labels
    #[arg(long, value_enum)]
    bar_number_format: Option<NumberFormat>,

    /// Hide labels (values/names) attached to the bars
    #[arg(long)]
    hide_bar_labels: bool,

    /// Disable the progress bar during GIF rendering
    #[arg(long)]
    no_progress: bool,

    /// Hide the chart title
    #[arg(long)]
    hide_title: bool,

    /// Font size for the chart title (default: 32)
    #[arg(long)]
    title_size: Option<u32>,

    /// Font size for bar labels (names and values, default: 15 for horizontal, 13 for vertical)
    #[arg(long)]
    bar_label_size: Option<u32>,

    /// Font size for the period / time label in the corner (default: 52)
    #[arg(long)]
    period_label_size: Option<u32>,

    /// Font size for axis tick numbers and axis descriptions (default: 12)
    #[arg(long)]
    axis_label_size: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Loading data from {:?}...", args.input);
    let data = TimeSeriesData::from_csv(&args.input)?;

    let effective_top_n = if args.top_n > data.categories.len() {
        println!(
            "Dataset has {} categories; automatically adjusting top_n from {} to {}.",
            data.categories.len(),
            args.top_n,
            data.categories.len()
        );
        data.categories.len()
    } else {
        args.top_n
    };

    let config = ChartConfig {
        width: args.width,
        height: args.height,
        fps: args.fps,
        steps_per_period: args.steps,
        top_n: effective_top_n,
        title: args.title,
        x_label: args.x_label,
        y_label: args.y_label,
        max_x: args.max_x,
        clamp_exit: args.clamp_exit,
        ascending: args.ascending,
        label_position: args.label_position,
        orientation: args.orientation,
        log_scale: args.log_scale,
        log_min: args.log_min,
        number_format: args.number_format,
        axis_number_format: args.axis_number_format,
        bar_number_format: args.bar_number_format,
        hide_bar_labels: args.hide_bar_labels,
        hide_title: args.hide_title,
        title_size: args.title_size,
        bar_label_size: args.bar_label_size,
        period_label_size: args.period_label_size,
        axis_label_size: args.axis_label_size,
    };

    println!(
        "Interpolating frames (steps: {}, top: {}, order: {})...",
        args.steps,
        config.top_n,
        if config.ascending {
            "ascending"
        } else {
            "descending"
        }
    );
    let frames = Interpolator::generate_frames_from_config(&data, &config);
    println!("Total frames generated: {}", frames.len());

    println!("Rendering GIF to {:?}...", args.output);
    let pb = if args.no_progress {
        indicatif::ProgressBar::hidden()
    } else {
        let pb = indicatif::ProgressBar::new(frames.len() as u64);
        if let Ok(style) = indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) [ETA: {eta}]")
        {
            pb.set_style(style.progress_chars("#>-"));
        }
        pb
    };

    let renderer = Renderer::new(config);
    renderer.render_gif_with_progress(&frames, &args.output, |current, _total| {
        pb.set_position(current as u64);
    })?;
    pb.finish();

    println!("Done! Saved to {:?}", args.output);
    Ok(())
}
