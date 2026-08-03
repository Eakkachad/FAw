//! External data binding (`katSVG Data Binding`).
//!
//! Binds data from external files (CSV / JSON) into a spec: metrics, chart
//! series, and section titles. The prompt supplies layout/theme/aspect; the
//! data file supplies values — nothing is invented. D4.
//!
//! Supported formats:
//! - CSV: header row + data rows. `label,value` → chart series;
//!   `key,value` (metric column names) → metrics.
//! - JSON: object of `label: number` or array of `{label, value}`.

use crate::router::{ChartSpec, ChartType, MetricCardSpec, SectionSpec};
use serde::Deserialize;

/// Data extracted from an external file.
#[derive(Debug, Clone, Default)]
pub struct BoundData {
    pub metrics: Vec<MetricCardSpec>,
    pub sections: Vec<SectionSpec>,
    pub chart: Option<ChartSpec>,
}

/// Parse a data file by extension (`csv` or `json`). Returns bound data.
pub fn parse_data(content: &str, path_or_ext: &str) -> Result<BoundData, String> {
    let ext = path_or_ext
        .split('.')
        .next_back()
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "csv" => parse_csv(content),
        "json" => parse_json(content),
        _ => Err(format!(
            "unsupported data format: {ext} (use .csv or .json)"
        )),
    }
}

fn parse_csv(content: &str) -> Result<BoundData, String> {
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or("empty CSV")?;
    let cols: Vec<&str> = header.split(',').map(str::trim).collect();
    if cols.len() < 2 {
        return Err("CSV needs at least 2 columns".to_string());
    }

    let mut labels = Vec::new();
    let mut rows: Vec<Vec<f64>> = Vec::new();

    for line in lines {
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        if cells.len() < 2 {
            continue;
        }
        let label = cells[0].to_string();
        let mut vals = Vec::with_capacity(cells.len() - 1);
        for c in &cells[1..] {
            let v = c
                .parse::<f64>()
                .map_err(|_| format!("non-numeric value in row: {line}"))?;
            vals.push(v);
        }
        labels.push(label);
        rows.push(vals);
    }

    // Build series: one per numeric column beyond the label column.
    let n_cols = cols.len() - 1;
    let mut series: Vec<Vec<f64>> = (0..n_cols).map(|_| Vec::new()).collect();
    for row in &rows {
        for (s, v) in series.iter_mut().zip(row.iter()) {
            s.push(*v);
        }
    }
    // Primary `values` = first series; extras go into `series`.
    let primary = series.first().cloned().unwrap_or_default();
    let extras: Vec<Vec<f64>> = series.into_iter().skip(1).collect();
    let series_names: Vec<String> = cols[1..].iter().map(|c| c.to_string()).collect();

    let mut out = BoundData::default();
    if labels.len() >= 2 {
        out.chart = Some(ChartSpec {
            chart_type: ChartType::Bar,
            labels,
            values: primary,
            unit: None,
            series: extras,
            series_names: if series_names.len() > 1 {
                series_names[1..].to_vec()
            } else {
                Vec::new()
            },
        });
    }
    let _ = cols.len();
    Ok(out)
}

#[derive(Deserialize)]
struct JsonEntry {
    #[serde(rename = "label", default)]
    label: String,
    #[serde(rename = "value", default)]
    value: f64,
    #[serde(rename = "name", default)]
    name: Option<String>,
    #[serde(rename = "amount", default)]
    amount: Option<f64>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

fn parse_json(content: &str) -> Result<BoundData, String> {
    let v: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("invalid JSON: {e}"))?;

    let mut out = BoundData::default();

    match v {
        serde_json::Value::Object(map) => {
            // { "label": number, ... } → chart series
            let mut labels = Vec::new();
            let mut values = Vec::new();
            let mut metrics = Vec::new();
            for (k, val) in map {
                if let Some(num) = val.as_f64() {
                    labels.push(k.clone());
                    values.push(num);
                } else if let Some(s) = val.as_str() {
                    metrics.push(MetricCardSpec {
                        label: k.to_uppercase(),
                        value: s.to_string(),
                        icon: "chart".to_string(),
                    });
                }
            }
            if labels.len() >= 2 {
                out.chart = Some(ChartSpec {
                    chart_type: ChartType::Bar,
                    labels,
                    values,
                    unit: None,
                    series: Vec::new(),
                    series_names: Vec::new(),
                });
            }
            if !metrics.is_empty() {
                out.metrics = metrics;
            }
        }
        serde_json::Value::Array(items) => {
            let mut labels = Vec::new();
            let mut values = Vec::new();
            let mut metrics = Vec::new();
            for item in items {
                let entry: JsonEntry =
                    serde_json::from_value(item).map_err(|e| format!("bad array item: {e}"))?;
                let label = if !entry.label.is_empty() {
                    entry.label
                } else {
                    entry.name.unwrap_or_default()
                };
                let value = if entry.value != 0.0 {
                    entry.value
                } else {
                    entry.amount.unwrap_or(0.0)
                };
                if !label.is_empty() {
                    labels.push(label);
                    values.push(value);
                }
                let _ = &entry.extra;
            }
            if labels.len() >= 2 {
                out.chart = Some(ChartSpec {
                    chart_type: ChartType::Bar,
                    labels,
                    values,
                    unit: None,
                    series: Vec::new(),
                    series_names: Vec::new(),
                });
            } else if !labels.is_empty() {
                metrics.push(MetricCardSpec {
                    label: labels[0].to_uppercase(),
                    value: format!("{}", values[0]),
                    icon: "chart".to_string(),
                });
            }
            let _ = &metrics;
        }
        _ => return Err("JSON must be an object or array".to_string()),
    }
    Ok(out)
}
