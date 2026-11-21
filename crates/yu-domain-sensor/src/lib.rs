//! 屿 · 系统指标。
//!
//! Linux 读 /proc；Windows 走 platform 层（由宿主注入），云端只保留桩。
//! 采样 1Hz，内核侧做合并推送；空闲时降到 0.2Hz。

#![forbid(unsafe_code)]

use std::path::Path;
use yu_abi::{Metric, MetricKind, Module, ModuleCtx, ModuleId, Source, Taxon, YuError};

#[derive(Default)]
pub struct Sensor {
    last: std::sync::Mutex<Vec<Metric>>,
}

impl Sensor {
    pub fn sample(&self) -> Vec<Metric> {
        let mut out = Vec::new();
        if let Some(cpu) = read_cpu_percent() {
            out.push(Metric {
                kind: MetricKind::CpuPercent,
                value: cpu,
                at_ms: yu_abi::now_ms(),
            });
        }
        if let Some((used_pct, used_mb)) = read_mem() {
            out.push(Metric {
                kind: MetricKind::MemPercent,
                value: used_pct,
                at_ms: yu_abi::now_ms(),
            });
            out.push(Metric {
                kind: MetricKind::MemUsedMb,
                value: used_mb,
                at_ms: yu_abi::now_ms(),
            });
        }
        *self.last.lock().unwrap() = out.clone();
        out
    }

    pub fn last(&self) -> Vec<Metric> {
        self.last.lock().unwrap().clone()
    }
}

impl Module for Sensor {
    fn id(&self) -> ModuleId {
        ModuleId::new("sensor")
    }

    fn taxon(&self) -> Taxon {
        Taxon {
            phylum: "sensus".into(),
            class: "system".into(),
            order: "metric".into(),
            family: "local".into(),
            genus: "procfs".into(),
            species: "sample".into(),
        }
    }

    fn capabilities(&self) -> &'static [yu_abi::Capability] {
        &[]
    }

    fn boot(&self, _ctx: &ModuleCtx) -> Result<(), YuError> {
        Ok(())
    }
}

impl Source<Metric> for Sensor {
    fn poll(&self) -> Result<Vec<Metric>, YuError> {
        let s = self.sample();
        if s.is_empty() {
            return Err(YuError::PlatformUnsupported {
                what: "sensor.sample".into(),
            });
        }
        Ok(s)
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_percent() -> Option<f32> {
    let raw = std::fs::read_to_string("/proc/stat").ok()?;
    let line = raw.lines().next()?;
    let mut it = line.split_whitespace();
    it.next();
    let nums: Vec<f64> = it.take(8).filter_map(|x| x.parse().ok()).collect();
    if nums.len() < 4 {
        return None;
    }
    let idle = nums[3] + nums.get(4).copied().unwrap_or(0.0);
    let total: f64 = nums.iter().take(8).sum();
    if total <= 0.0 {
        return None;
    }
    Some((((total - idle) / total) * 100.0) as f32)
}

#[cfg(not(target_os = "linux"))]
fn read_cpu_percent() -> Option<f32> {
    None
}

#[cfg(target_os = "linux")]
fn read_mem() -> Option<(f32, f32)> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total_kb = v.split_whitespace().next()?.parse().ok()?;
        }
        if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail_kb = v.split_whitespace().next()?.parse().ok()?;
        }
    }
    if total_kb == 0 {
        return None;
    }
    let used_kb = total_kb.saturating_sub(avail_kb);
    Some((
        (used_kb as f32 / total_kb as f32) * 100.0,
        (used_kb as f32) / 1024.0,
    ))
}

#[cfg(not(target_os = "linux"))]
fn read_mem() -> Option<(f32, f32)> {
    None
}

/// 读取 sysfs 电池（Linux 笔记本）。
pub fn battery_percent() -> Option<f32> {
    let p = Path::new("/sys/class/power_supply/BAT0/capacity");
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
