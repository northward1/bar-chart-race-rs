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
        }
    }
}
