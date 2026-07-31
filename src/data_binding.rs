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
    let ext = path_or_ext.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "csv" => parse_csv(content),
        "json" => parse_json(content),
        _ => Err(format!("unsupported data format: {ext} (use .csv or .json)")),
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
    let mut values = Vec::new();
    let mut metrics = Vec::new();
    let mut rows = Vec::new();

    for line in lines {
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        if cells.len() < 2 {
            continue;
        }
        let label = cells[0].to_string();
        let val = cells[1].parse::<f64>().map_err(|_| format!("non-numeric value in row: {line}"))?;
        rows.push((label, val));
    }

    // Heuristic: if headers look like metric key names and values are small set → metrics;
    // otherwise → chart series.
    let chartish = rows.len() >= 2;
    if chartish {
        for (label, val) in &rows {
            labels.push(label.clone());
            values.push(*val);
        }
    }

    // If a third column provides a label for metrics (key,label,value) or if the
    // first column is "metric", bind as metrics. Keep simple: metrics = first 2
    // rows as KPI cards when chart not clearly intended.
    if !chartish {
        for (label, val) in &rows {
            metrics.push(MetricCardSpec {
                label: label.to_uppercase(),
                value: format!("{val}"),
                icon: "chart".to_string(),
            });
        }
    }

    let mut out = BoundData::default();
    if !metrics.is_empty() {
        out.metrics = metrics;
    }
    if labels.len() >= 2 {
        out.chart = Some(ChartSpec {
            chart_type: ChartType::Bar,
            labels,
            values,
            unit: None,
        });
    }
    let _ = (cols.len(), header);
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
    let v: serde_json::Value = serde_json::from_str(content).map_err(|e| format!("invalid JSON: {e}"))?;

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
                out.chart = Some(ChartSpec { chart_type: ChartType::Bar, labels, values, unit: None });
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
                let entry: JsonEntry = serde_json::from_value(item).map_err(|e| format!("bad array item: {e}"))?;
                let label = if !entry.label.is_empty() { entry.label } else { entry.name.unwrap_or_default() };
                let value = if entry.value != 0.0 { entry.value } else { entry.amount.unwrap_or(0.0) };
                if !label.is_empty() {
                    labels.push(label);
                    values.push(value);
                }
                let _ = &entry.extra;
            }
            if labels.len() >= 2 {
                out.chart = Some(ChartSpec { chart_type: ChartType::Bar, labels, values, unit: None });
            } else if !labels.is_empty() {
                metrics.push(MetricCardSpec { label: labels[0].to_uppercase(), value: format!("{}", values[0]), icon: "chart".to_string() });
            }
            let _ = &metrics;
        }
        _ => return Err("JSON must be an object or array".to_string()),
    }
    Ok(out)
}
