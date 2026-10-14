use bar_chart_race::{ChartConfig, Interpolator, Renderer, TimeSeriesData};
use tempfile::NamedTempFile;

#[test]
fn test_interpolation_and_rendering() {
    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");
    assert_eq!(data.periods.len(), 5);
    assert_eq!(data.categories.len(), 5);

    let config = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 2,
        top_n: 3,
        title: "Test".to_string(),
        ..Default::default()
    };

    let frames = Interpolator::generate_frames(&data, config.steps_per_period, config.top_n);
    assert!(!frames.is_empty());

    let tmp_file = NamedTempFile::new().expect("Create temp file");
    let renderer = Renderer::new(config);
    renderer
        .render_gif(&frames, tmp_file.path())
        .expect("Rendering should succeed");

    let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
    assert!(metadata.len() > 0);
}

#[test]
fn test_ascending_order_and_custom_options() {
    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    let config = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 2,
        top_n: 3,
        title: "Test Ascending".to_string(),
        x_label: Some("Custom Value".to_string()),
        y_label: Some("City".to_string()),
        max_x: Some(50.0),
        clamp_exit: false,
        ascending: true,
        ..Default::default()
    };

    let frames = Interpolator::generate_frames_from_config(&data, &config);
    assert!(!frames.is_empty());

    // In ascending order, smallest value should be ranked first (rank 0)
    let first_frame = &frames[0];
    assert!(!first_frame.bars.is_empty());
    assert!(first_frame.bars[0].rank < first_frame.bars[1].rank);
    assert!(first_frame.bars[0].value <= first_frame.bars[1].value);

    let tmp_file = NamedTempFile::new().expect("Create temp file");
    let renderer = Renderer::new(config);
    renderer
        .render_gif(&frames, tmp_file.path())
        .expect("Rendering should succeed");

    let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
    assert!(metadata.len() > 0);
}

#[test]
fn test_label_positions() {
    use bar_chart_race::LabelPosition;

    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    for position in [LabelPosition::YAxis, LabelPosition::BarTip] {
        let config = ChartConfig {
            width: 400,
            height: 300,
            fps: 10,
            steps_per_period: 2,
            top_n: 3,
            label_position: position,
            ..Default::default()
        };

        let frames = Interpolator::generate_frames_from_config(&data, &config);
        let tmp_file = NamedTempFile::new().expect("Create temp file");
        let renderer = Renderer::new(config);
        renderer
            .render_gif(&frames, tmp_file.path())
            .expect("Rendering should succeed");

        let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
        assert!(metadata.len() > 0);
    }
}

#[test]
fn test_orientations() {
    use bar_chart_race::Orientation;

    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let config = ChartConfig {
            width: 400,
            height: 300,
            fps: 10,
            steps_per_period: 2,
            top_n: 3,
            orientation,
            ..Default::default()
        };

        let frames = Interpolator::generate_frames_from_config(&data, &config);
        let tmp_file = NamedTempFile::new().expect("Create temp file");
        let renderer = Renderer::new(config);
        renderer
            .render_gif(&frames, tmp_file.path())
            .expect("Rendering should succeed");

        let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
        assert!(metadata.len() > 0);
    }
}

#[test]
fn test_log_scale() {
    use bar_chart_race::Orientation;

    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let config = ChartConfig {
            width: 400,
            height: 300,
            fps: 10,
            steps_per_period: 2,
            top_n: 3,
            orientation,
            log_scale: true,
            log_min: 0.5,
            ..Default::default()
        };

        let frames = Interpolator::generate_frames_from_config(&data, &config);
        let tmp_file = NamedTempFile::new().expect("Create temp file");
        let renderer = Renderer::new(config);
        renderer
            .render_gif(&frames, tmp_file.path())
            .expect("Rendering should succeed");

        let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
        assert!(metadata.len() > 0);
    }
}
