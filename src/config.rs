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
        }
    }
}
