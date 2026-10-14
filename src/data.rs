use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

/// Represents time-series entry: date/period -> map of (item_name -> value)
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    pub periods: Vec<String>,
    pub values: BTreeMap<String, Vec<f64>>,
    pub categories: Vec<String>,
}

impl TimeSeriesData {
    /// Load time series data from a wide-format CSV file (first column is date/time, remaining columns are categories)
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path).context("Failed to open CSV file")?;
        let headers = reader.headers()?.clone();

        let categories: Vec<String> = headers
            .iter()
            .skip(1)
            .map(|s| s.trim().to_string())
            .collect();

        let mut periods = Vec::new();
        let mut values_map: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for cat in &categories {
            values_map.insert(cat.clone(), Vec::new());
        }

        for result in reader.records() {
            let record = result?;
            if record.is_empty() {
                continue;
            }
            let period = record.get(0).unwrap_or("").trim().to_string();
            periods.push(period);

            for (i, cat) in categories.iter().enumerate() {
                let val: f64 = record
                    .get(i + 1)
                    .unwrap_or("0")
                    .trim()
                    .parse()
                    .unwrap_or(0.0);
                values_map.get_mut(cat).unwrap().push(val);
            }
        }

        Ok(Self {
            periods,
            values: values_map,
            categories,
        })
    }
}
