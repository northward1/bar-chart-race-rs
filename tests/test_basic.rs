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

#[test]
fn test_auto_clamp_top_n() {
    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");
    assert_eq!(data.categories.len(), 5);

    // Request top_n = 20, should clamp to 5 categories
    let config = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 2,
        top_n: 20,
        ..Default::default()
    };

    let frames = Interpolator::generate_frames_from_config(&data, &config);
    assert!(!frames.is_empty());
    // Max bars in any frame should not exceed data.categories.len() (5)
    for frame in &frames {
        assert!(frame.bars.len() <= 5);
        for bar in &frame.bars {
            assert!(bar.rank < 5.0);
        }
    }
}

#[test]
fn test_number_formats() {
    use bar_chart_race::NumberFormat;

    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    for fmt in [
        NumberFormat::Integer,
        NumberFormat::Compact,
        NumberFormat::Scientific,
        NumberFormat::Exp,
        NumberFormat::Fixed1,
        NumberFormat::Fixed2,
    ] {
        let config = ChartConfig {
            width: 400,
            height: 300,
            fps: 10,
            steps_per_period: 2,
            top_n: 3,
            number_format: fmt,
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
fn test_hide_bar_labels() {
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
            hide_bar_labels: true,
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
fn test_axis_and_bar_number_formats() {
    use bar_chart_race::NumberFormat;

    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    let config = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 2,
        top_n: 3,
        number_format: NumberFormat::Compact,
        axis_number_format: Some(NumberFormat::Exp),
        bar_number_format: Some(NumberFormat::Fixed2),
        ..Default::default()
    };

    assert_eq!(config.effective_axis_format(), NumberFormat::Exp);
    assert_eq!(config.effective_bar_format(), NumberFormat::Fixed2);

    let frames = Interpolator::generate_frames_from_config(&data, &config);
    let tmp_file = NamedTempFile::new().expect("Create temp file");
    let renderer = Renderer::new(config);
    renderer
        .render_gif(&frames, tmp_file.path())
        .expect("Rendering should succeed");

    let metadata = std::fs::metadata(tmp_file.path()).expect("File metadata");
    assert!(metadata.len() > 0);
}

#[test]
fn test_render_gif_with_progress() {
    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    let config = ChartConfig {
        width: 300,
        height: 200,
        fps: 10,
        steps_per_period: 1,
        top_n: 3,
        title: "Progress Test".to_string(),
        ..Default::default()
    };

    let frames = Interpolator::generate_frames_from_config(&data, &config);
    let tmp_file = NamedTempFile::new().expect("Create temp file");
    let renderer = Renderer::new(config);

    let mut progress_calls = Vec::new();
    renderer
        .render_gif_with_progress(&frames, tmp_file.path(), |current, total| {
            progress_calls.push((current, total));
        })
        .expect("Rendering should succeed");

    assert_eq!(progress_calls.len(), frames.len());
    for (i, &(current, total)) in progress_calls.iter().enumerate() {
        assert_eq!(current, i + 1);
        assert_eq!(total, frames.len());
    }
}

#[test]
fn test_hide_title_and_font_sizes() {
    let data =
        TimeSeriesData::from_csv("examples/data/cities.csv").expect("CSV loading should succeed");

    // Test horizontal with hidden title and custom font sizes
    let config_h = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 1,
        top_n: 3,
        title: "Hidden Title".to_string(),
        hide_title: true,
        title_size: Some(24),
        bar_label_size: Some(10),
        period_label_size: Some(30),
        axis_label_size: Some(10),
        x_label: Some("X Axis".to_string()),
        y_label: Some("Y Axis".to_string()),
        ..Default::default()
    };
    assert!(!config_h.is_title_visible());
    assert_eq!(config_h.effective_title_size(), 24);
    assert_eq!(config_h.effective_bar_label_size(), 10);
    assert_eq!(config_h.effective_period_label_size(), 30);
    assert_eq!(config_h.effective_axis_label_size(), 10);

    let frames_h = Interpolator::generate_frames_from_config(&data, &config_h);
    let tmp_file_h = NamedTempFile::new().expect("Create temp file");
    let renderer_h = Renderer::new(config_h);
    renderer_h
        .render_gif(&frames_h, tmp_file_h.path())
        .expect("Horizontal rendering with hidden title should succeed");
    assert!(std::fs::metadata(tmp_file_h.path()).unwrap().len() > 0);

    // Test vertical with empty title (should also treat as hidden) and custom font sizes
    let config_v = ChartConfig {
        width: 400,
        height: 300,
        fps: 10,
        steps_per_period: 1,
        top_n: 3,
        title: "   ".to_string(), // whitespace-only title
        orientation: bar_chart_race::Orientation::Vertical,
        bar_label_size: Some(11),
        period_label_size: Some(36),
        axis_label_size: Some(9),
        ..Default::default()
    };
    assert!(!config_v.is_title_visible());

    let frames_v = Interpolator::generate_frames_from_config(&data, &config_v);
    let tmp_file_v = NamedTempFile::new().expect("Create temp file");
    let renderer_v = Renderer::new(config_v);
    renderer_v
        .render_gif(&frames_v, tmp_file_v.path())
        .expect("Vertical rendering with empty title should succeed");
    assert!(std::fs::metadata(tmp_file_v.path()).unwrap().len() > 0);
}
