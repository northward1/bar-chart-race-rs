#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub steps_per_period: usize,
    pub top_n: usize,
    pub title: String,
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
        }
    }
}

