use crate::config::ChartConfig;
use crate::data::TimeSeriesData;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BarState {
    pub name: String,
    pub value: f64,
    pub rank: f64, // float rank: 0.0 is top (1st place), top_n is off screen
    pub color_idx: usize,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub period_label: String,
    pub bars: Vec<BarState>,
    pub max_value: f64,
}

pub struct Interpolator;

impl Interpolator {
    /// Generate frames using explicit parameters (backwards compatible)
    pub fn generate_frames(
        data: &TimeSeriesData,
        steps_per_period: usize,
        top_n: usize,
    ) -> Vec<Frame> {
        let config = ChartConfig {
            steps_per_period,
            top_n,
            ..Default::default()
        };
        Self::generate_frames_from_config(data, &config)
    }

    /// Generate frames according to full ChartConfig (supporting ascending sort and exit animations)
    pub fn generate_frames_from_config(data: &TimeSeriesData, config: &ChartConfig) -> Vec<Frame> {
        let n_periods = data.periods.len();
        if n_periods == 0 {
            return Vec::new();
        }

        let steps_per_period = config.steps_per_period;
        let top_n = config.top_n.min(data.categories.len());

        // Calculate discrete ranking for each category at each period
        let mut period_ranks: Vec<HashMap<String, usize>> = Vec::with_capacity(n_periods);
        for p in 0..n_periods {
            let mut cat_vals: Vec<(&String, f64)> = data
                .categories
                .iter()
                .map(|cat| {
                    let val = data
                        .values
                        .get(cat)
                        .and_then(|v| v.get(p))
                        .copied()
                        .unwrap_or(0.0);
                    (cat, val)
                })
                .collect();

            if config.ascending {
                cat_vals.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                cat_vals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }

            let mut ranks = HashMap::new();
            for (rank, (cat, _)) in cat_vals.into_iter().enumerate() {
                ranks.insert((*cat).clone(), rank);
            }
            period_ranks.push(ranks);
        }

        // Persistent category color mapping:
        // Prioritize categories that actually appear in top_n across periods
        let mut visible_order: Vec<String> = Vec::new();
        for ranks in &period_ranks {
            let mut sorted_period: Vec<(&String, &usize)> = ranks.iter().collect();
            sorted_period.sort_by_key(|&(_, r)| *r);
            for (cat, &r) in sorted_period {
                if r < top_n && !visible_order.contains(cat) {
                    visible_order.push((*cat).clone());
                }
            }
        }

        let mut color_map: HashMap<String, usize> = HashMap::new();
        for (idx, cat) in visible_order.iter().enumerate() {
            color_map.insert(cat.clone(), idx);
        }
        let mut next_idx = visible_order.len();
        for cat in &data.categories {
            if !color_map.contains_key(cat) {
                color_map.insert(cat.clone(), next_idx);
                next_idx += 1;
            }
        }

        let mut frames = Vec::new();
        let fallback_rank = data.categories.len();

        for i in 0..n_periods {
            let next_i = (i + 1).min(n_periods - 1);
            let steps = if i == n_periods - 1 {
                1
            } else {
                steps_per_period
            };

            let ranks_0 = &period_ranks[i];
            let ranks_1 = &period_ranks[next_i];

            for step in 0..steps {
                let t = if steps > 1 {
                    step as f64 / steps as f64
                } else {
                    0.0
                };

                // Smooth easing curve (easeInOutCubic) for ranking transition
                let ease_t = if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                };

                let mut frame_bars: Vec<BarState> = Vec::new();
                let mut max_value: f64 = 1.0;

                for cat in &data.categories {
                    let v0 = data
                        .values
                        .get(cat)
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or(0.0);
                    let v1 = data
                        .values
                        .get(cat)
                        .and_then(|v| v.get(next_i))
                        .copied()
                        .unwrap_or(v0);
                    let val = v0 + (v1 - v0) * t;

                    let r0 = ranks_0.get(cat).copied().unwrap_or(fallback_rank) as f64;
                    let r1 = ranks_1.get(cat).copied().unwrap_or(fallback_rank) as f64;

                    // If both periods are outside top_n, skip
                    if r0 >= top_n as f64 && r1 >= top_n as f64 {
                        continue;
                    }

                    // Exit animation: clamped to top_n (old behavior) or actual ranks (new default)
                    let (start_r, end_r) = if config.clamp_exit {
                        (r0.min(top_n as f64), r1.min(top_n as f64))
                    } else {
                        (r0, r1)
                    };
                    let current_rank = start_r + (end_r - start_r) * ease_t;

                    // Only keep bars within or slightly below visible range
                    if current_rank <= (top_n + 2) as f64 {
                        if current_rank < top_n as f64 && val > max_value {
                            max_value = val;
                        }

                        frame_bars.push(BarState {
                            name: cat.clone(),
                            value: val,
                            rank: current_rank,
                            color_idx: *color_map.get(cat).unwrap_or(&0),
                        });
                    }
                }

                // Sort bars so they are ordered by rank
                frame_bars.sort_by(|a, b| {
                    a.rank
                        .partial_cmp(&b.rank)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                frames.push(Frame {
                    period_label: data.periods[i].clone(),
                    bars: frame_bars,
                    max_value,
                });
            }
        }

        frames
    }
}
