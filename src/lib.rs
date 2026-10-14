pub mod config;
pub mod data;
pub mod interpolation;
pub mod renderer;

pub use config::ChartConfig;
pub use data::TimeSeriesData;
pub use interpolation::{Frame, Interpolator};
pub use renderer::Renderer;

