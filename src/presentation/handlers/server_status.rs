use axum::{response::IntoResponse, Json};
use serde_json::json;
use std::fs;

pub async fn server_status_handler() -> impl IntoResponse {
    let meminfo = read_meminfo();
    let uptime = read_uptime();
    let loadavg = read_loadavg();
    Json(json!({
        "memory": meminfo,
        "uptime_seconds": uptime,
        "load_average": loadavg,
    }))
}

fn read_meminfo() -> serde_json::Value {
    let mut total = 0u64;
    let mut available = 0u64;
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kb(line);
            } else if line.starts_with("MemAvailable:") {
                available = parse_kb(line);
            }
        }
    }
    let used = total.saturating_sub(available);
    json!({
        "total_kb": total,
        "available_kb": available,
        "used_kb": used,
    })
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

fn read_uptime() -> f64 {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .unwrap_or(0.0)
}

fn read_loadavg() -> serde_json::Value {
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return json!({
                "1min": parts[0].parse::<f64>().unwrap_or(0.0),
                "5min": parts[1].parse::<f64>().unwrap_or(0.0),
                "15min": parts[2].parse::<f64>().unwrap_or(0.0),
            });
        }
    }
    json!({})
}