use bar_chart_race::{ChartConfig, Interpolator, Renderer, TimeSeriesData};
use tempfile::NamedTempFile;

#[test]
fn test_interpolation_and_rendering() {
    let data = TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");
    assert_eq!(data.periods.len(), 5);
    assert_eq!(data.categories.len(), 5);

    let config = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 2,
        top_n: 3,
        title: "Test".to_string(),
    };

    let frames = Interpolator::generate_frames(&data, config.steps_per_period, config.top_n);
    assert!(!frames.is_empty());

    let tmp_file = NamedTempFile::new().expect("Create temp file");
    let renderer = Renderer::new(config);
    renderer.render_gif(&frames, tmp_file.path()).expect("Rendering should succeed");

    let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
    assert!(metadata.len() > 0);
}
