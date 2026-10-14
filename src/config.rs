use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
pub enum LabelPosition {
    /// Draw category name beside the Y-axis (left of bar) and value at the bar tip
    #[default]
    #[value(name = "y-axis")]
    YAxis,
    /// Draw category name and value together at the bar tip
    #[value(name = "bar-tip")]
    BarTip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
pub enum Orientation {
    /// Horizontal bars (bars extend left-to-right, ranks ordered top-to-bottom)
    #[default]
    #[value(name = "horizontal")]
    Horizontal,
    /// Vertical bars (bars extend bottom-to-top, ranks ordered left-to-right)
    #[value(name = "vertical")]
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
pub enum NumberFormat {
    /// Automatic formatting (e.g. integer for >=1000, 1 decimal for >=10, 2 decimals otherwise)
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Integer only (e.g. 1000, 42)
    #[value(name = "integer")]
    Integer,
    /// Compact notation with SI units (e.g. 1.5B, 20.3M, 450K)
    #[value(name = "compact")]
    Compact,
    /// Scientific notation (e.g. 1.5e9, 2.0e7)
    #[value(name = "scientific")]
    Scientific,
    /// Exponential notation with superscripts (e.g. 10⁹, 1.5×10⁹)
    #[value(name = "exp")]
    Exp,
    /// Fixed 1 decimal place (e.g. 1000.0, 42.5)
    #[value(name = "fixed1")]
    Fixed1,
    /// Fixed 2 decimal places (e.g. 1000.00, 42.50)
    #[value(name = "fixed2")]
    Fixed2,
}

#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub steps_per_period: usize,
    pub top_n: usize,
    pub title: String,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub max_x: Option<f64>,
    pub clamp_exit: bool,
    pub ascending: bool,
    pub label_position: LabelPosition,
    pub orientation: Orientation,
    pub log_scale: bool,
    pub log_min: f64,
    pub number_format: NumberFormat,
    pub axis_number_format: Option<NumberFormat>,
    pub bar_number_format: Option<NumberFormat>,
    pub hide_bar_labels: bool,
}

impl ChartConfig {
    pub fn effective_axis_format(&self) -> NumberFormat {
        self.axis_number_format.unwrap_or(self.number_format)
    }

    pub fn effective_bar_format(&self) -> NumberFormat {
        self.bar_number_format.unwrap_or(self.number_format)
    }
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            fps: 30,
            steps_per_period: 20,
            top_n: 10,
            title: "Bar Chart Race".to_string(),
            x_label: None,
            y_label: None,
            max_x: None,
            clamp_exit: false,
            ascending: false,
            label_position: LabelPosition::default(),
            orientation: Orientation::default(),
            log_scale: false,
            log_min: 1.0,
            number_format: NumberFormat::default(),
            axis_number_format: None,
            bar_number_format: None,
            hide_bar_labels: false,
        }
    }
}
