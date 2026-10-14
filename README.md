# bar-chart-race-rs

A fast, lightweight, and customizable **Bar Chart Race** generator in Rust using [Plotters](https://github.com/plotters-rs/plotters).

Generates animated Bar Chart Race visualizations (e.g., animated GIF) directly from time-series CSV data.

## Features

- **Pure Rust Rendering**: Uses Plotters bitmap backend and image encoder (no external Python or JavaScript runtime required).
- **Smooth Ranking & Value Interpolation**: Cubic easing (`easeInOutCubic`) ensures bars glide smoothly as they overtake each other.
- **Entry & Exit Animations**: Bars dropping below or rising into the top-N slide smoothly in and out from the bottom edge.
- **Persistent Category Colors**: Each entity keeps a consistent color throughout ranking swaps.
- **CLI & Library**: Use as a standalone command-line tool or integrate into your own Rust project.

## Demo

![World Urban Population Demo](https://raw.githubusercontent.com/northward1/bar-chart-race-rs/main/race_urban.gif)

## Installation

### From crates.io
```bash
cargo install bar-chart-race
```

### From source
```bash
git clone https://github.com/northward1/bar-chart-race-rs.git
cd bar-chart-race-rs
cargo build --release
```

## CLI Usage

```bash
# Example with World Urban Population dataset:
bar-chart-race \
  --input examples/data/urban_pop.csv \
  --output race_urban.gif \
  --title "World Urban Population (1976-2018)" \
  --x-label "Urban Population" \
  --number-format exp \
  --top-n 10 \
  --width 800 \
  --height 500 \
  --fps 12 \
  --steps 3

# Example with Tech Giants Market Cap dataset:
bar-chart-race \
  --input examples/data/tech_market_cap.csv \
  --output race_tech.gif \
  --title "Top Global Companies by Market Cap (\$B)" \
  --x-label "Market Cap (\$B)" \
  --top-n 8 \
  --fps 20 \
  --steps 12
```

### Options

- `-i, --input <PATH>`: Input CSV file path (wide-format: 1st column is date/period, rest are categories)
- `-o, --output <PATH>`: Output GIF file path (default: `output.gif`)
- `-t, --title <STRING>`: Chart title (default: `Bar Chart Race`)
- `-n, --top-n <NUMBER>`: Number of top bars to display (default: `10`)
- `--fps <NUMBER>`: Frame rate for the output GIF (default: `30`)
- `-s, --steps <NUMBER>`: Number of interpolated intermediate frames between periods (default: `15`)
- `--width <PIXELS>`: Width of output image (default: `1024`)
- `--height <PIXELS>`: Height of output image (default: `768`)
- `--x-label <STRING>`: Description label for the X-axis (value axis)
- `--y-label <STRING>`: Description label for the Y-axis (category axis)
- `--max-x <FLOAT>`: Fixed upper limit for the X-axis
- `--ascending`: Sort in ascending order (smallest values at the top)
- `--clamp-exit`: Clamp exit/entry animations to the bottom edge instead of following actual rank trajectory
- `--label-position <y-axis|bar-tip>`: Position of bar labels: `y-axis` (beside Y axis with values at bar tips, default) or `bar-tip` (category name and value together at bar tips)
- `--orientation <horizontal|vertical>`: Orientation of bars: `horizontal` (default) or `vertical` (bars grow upward, ranks left-to-right)
- `--log-scale`: Use logarithmic scale for values
- `--log-min <FLOAT>`: Baseline and minimum positive value for logarithmic scale (default: `1.0`, values below are clamped to length 0)
- `--number-format <auto|integer|compact|scientific|exp|fixed1|fixed2>`: Formatting for values and axis ticks (e.g. `exp` for $10^9$ / $1.5\times 10^9$, default: `auto`)

## Sample Datasets

Located under [`examples/data/`](file:///home/ttokita/bar-chart-race-rs/examples/data/):
1. **`tech_market_cap.csv`**: Market cap ($B) of major companies (Apple, Microsoft, Nvidia, Amazon, GE, Cisco, etc.) from 2000 to 2024. Features intense ranking changes, new entrants (Apple, Alphabet, Nvidia surging from bottom to top), and legacy giants dropping off the screen.
2. **`urban_pop.csv`**: Historical urban population across 37 countries from 1976 to 2018.
3. **`cities.csv`**: Minimal quick-test dataset.

## Library Example

```rust
use bar_chart_race::{ChartConfig, Interpolator, Renderer, TimeSeriesData};

fn main() -> anyhow::Result<()> {
    let data = TimeSeriesData::from_csv("examples/data/tech_market_cap.csv")?;
    let config = ChartConfig {
        width: 1024,
        height: 768,
        fps: 20,
        steps_per_period: 12,
        top_n: 8,
        title: "Top Global Companies by Market Cap ($B)".to_string(),
        ..Default::default()
    };

    let frames = Interpolator::generate_frames(&data, config.steps_per_period, config.top_n);
    let renderer = Renderer::new(config);
    renderer.render_gif(&frames, "race_tech.gif")?;

    Ok(())
}
```

## Directory Structure

```text
.
├── Cargo.toml
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # CLI binary entry point
│   ├── config.rs           # Chart configuration options
│   ├── data.rs             # CSV parsing & time-series storage
│   ├── interpolation.rs    # Value and continuous ranking interpolation with easing
│   └── renderer.rs         # Plotters-based continuous Y-rendering & persistent colors
├── examples/
│   ├── generate_sample.rs  # Sample code
│   └── data/
│       ├── tech_market_cap.csv # Tech companies dataset (2000-2024)
│       ├── urban_pop.csv       # 37 countries urban pop (1976-2018)
│       └── cities.csv          # Minimal sample dataset
└── tests/
    └── test_basic.rs       # Integration test
```

## License

MIT