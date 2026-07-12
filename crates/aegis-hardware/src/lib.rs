//! Aegis Nexus hardware symbiosis: probe the host and derive throttle policy.
//!
//! Reads Linux `/proc` surfaces when present; degrades gracefully on missing
//! files so unit tests and non-Linux hosts still get a usable snapshot.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

/// Point-in-time view of the machine Aegis is running on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostSnapshot {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub cpus: u32,
    pub load_1: Option<f32>,
    pub mem_total_kb: u64,
    pub mem_avail_kb: u64,
    pub battery_pct: Option<u8>,
    pub thermal_hint: String,
    pub entropy_source: String,
    pub probed_at: DateTime<Utc>,
}

/// Runtime throttle knobs derived from a [`HostSnapshot`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThrottlePolicy {
    pub max_tool_parallel: usize,
    pub preferred_effort: String,
    pub max_agent_steps: usize,
    pub notes: String,
}

/// Probe the local host. Never panics; missing `/proc` files use fallbacks.
pub fn probe_host() -> HostSnapshot {
    let hostname = read_hostname();
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let cpus = probe_cpus();
    let load_1 = read_load_1();
    let (mem_total_kb, mem_avail_kb) = read_meminfo();
    let battery_pct = read_battery_pct();
    let thermal_hint = thermal_hint(load_1, mem_total_kb, mem_avail_kb, battery_pct);
    let entropy_source = if Path::new("/dev/urandom").exists() {
        "/dev/urandom".to_string()
    } else {
        "getrandom".to_string()
    };

    HostSnapshot {
        hostname,
        os,
        arch,
        cpus,
        load_1,
        mem_total_kb,
        mem_avail_kb,
        battery_pct,
        thermal_hint,
        entropy_source,
        probed_at: Utc::now(),
    }
}

/// Local heuristics: low free memory lowers parallel; high load lowers effort.
pub fn policy_from_snapshot(snap: &HostSnapshot) -> ThrottlePolicy {
    let mut notes: Vec<String> = Vec::new();
    let cpus = snap.cpus.max(1) as usize;

    // Free-memory ratio (avail / total). Missing totals → treat as healthy.
    let mem_ratio = if snap.mem_total_kb > 0 {
        snap.mem_avail_kb as f32 / snap.mem_total_kb as f32
    } else {
        1.0
    };

    let load = snap.load_1.unwrap_or(0.0);
    let load_per_cpu = load / cpus as f32;

    // Parallel tools: start from CPU count, clamp by memory pressure.
    let mut max_tool_parallel = cpus.clamp(1, 8);
    if mem_ratio < 0.10 {
        max_tool_parallel = 1;
        notes.push("low free memory → max_tool_parallel=1".into());
    } else if mem_ratio < 0.20 {
        max_tool_parallel = max_tool_parallel.min(2);
        notes.push("tight free memory → max_tool_parallel≤2".into());
    } else if mem_ratio < 0.35 {
        max_tool_parallel = max_tool_parallel.min(4);
        notes.push("moderate free memory → max_tool_parallel≤4".into());
    }

    // Effort: high load → lighter reasoning.
    let preferred_effort = if load_per_cpu >= 2.0 {
        notes.push("high load → preferred_effort=low".into());
        "low".to_string()
    } else if load_per_cpu >= 1.0 || mem_ratio < 0.15 {
        notes.push("elevated load or tight mem → preferred_effort=medium".into());
        "medium".to_string()
    } else {
        "high".to_string()
    };

    // Agent steps: shrink under resource pressure.
    let mut max_agent_steps = 48;
    if mem_ratio < 0.10 || load_per_cpu >= 2.0 {
        max_agent_steps = 16;
        notes.push("resource pressure → max_agent_steps=16".into());
    } else if mem_ratio < 0.25 || load_per_cpu >= 1.0 {
        max_agent_steps = 32;
        notes.push("mild pressure → max_agent_steps=32".into());
    }

    if let Some(pct) = snap.battery_pct {
        if pct <= 15 {
            max_tool_parallel = max_tool_parallel.min(1);
            max_agent_steps = max_agent_steps.min(16);
            notes.push(format!("battery {pct}% → conservative throttle"));
        } else if pct <= 30 {
            max_tool_parallel = max_tool_parallel.min(2);
            notes.push(format!("battery {pct}% → reduced parallel"));
        }
    }

    if notes.is_empty() {
        notes.push("host healthy → default throttle".into());
    }

    ThrottlePolicy {
        max_tool_parallel,
        preferred_effort,
        max_agent_steps,
        notes: notes.join("; "),
    }
}

/// Monochrome plain-text probe summary for CLI surfaces.
pub fn format_probe(snap: &HostSnapshot) -> String {
    let policy = policy_from_snapshot(snap);
    let load = snap
        .load_1
        .map(|l| format!("{l:.2}"))
        .unwrap_or_else(|| "n/a".into());
    let battery = snap
        .battery_pct
        .map(|p| format!("{p}%"))
        .unwrap_or_else(|| "n/a".into());
    let mem_total_mb = snap.mem_total_kb / 1024;
    let mem_avail_mb = snap.mem_avail_kb / 1024;

    format!(
        "\
aegis-hardware probe
  host      {hostname}
  os/arch   {os}/{arch}
  cpus      {cpus}
  load_1    {load}
  memory    {mem_avail_mb} / {mem_total_mb} MiB free/total
  battery   {battery}
  thermal   {thermal}
  entropy   {entropy}
  probed    {probed}
throttle
  parallel  {parallel}
  effort    {effort}
  steps     {steps}
  notes     {notes}
",
        hostname = snap.hostname,
        os = snap.os,
        arch = snap.arch,
        cpus = snap.cpus,
        load = load,
        mem_avail_mb = mem_avail_mb,
        mem_total_mb = mem_total_mb,
        battery = battery,
        thermal = snap.thermal_hint,
        entropy = snap.entropy_source,
        probed = snap.probed_at.to_rfc3339(),
        parallel = policy.max_tool_parallel,
        effort = policy.preferred_effort,
        steps = policy.max_agent_steps,
        notes = policy.notes,
    )
}

/// Overlay a [`ThrottlePolicy`] onto a JSON config value.
///
/// Merges into an object under key `"throttle"` (creating the object if needed).
/// Non-object roots become `{ "value": <root>, "throttle": {…} }`.
pub fn apply_policy_to_json(base: Value, policy: &ThrottlePolicy) -> Value {
    let throttle = json!({
        "max_tool_parallel": policy.max_tool_parallel,
        "preferred_effort": policy.preferred_effort,
        "max_agent_steps": policy.max_agent_steps,
        "notes": policy.notes,
    });

    match base {
        Value::Object(mut map) => {
            map.insert("throttle".into(), throttle);
            Value::Object(map)
        }
        other => {
            let mut map = Map::new();
            map.insert("value".into(), other);
            map.insert("throttle".into(), throttle);
            Value::Object(map)
        }
    }
}

// ── internal probes ─────────────────────────────────────────────────────────

fn read_hostname() -> String {
    if let Ok(h) = fs::read_to_string("/etc/hostname") {
        let t = h.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn probe_cpus() -> u32 {
    if let Some(n) = read_cpuinfo_processors() {
        return n.max(1);
    }
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
        .max(1)
}

fn read_cpuinfo_processors() -> Option<u32> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    let count = text.lines().filter(|l| l.starts_with("processor")).count() as u32;
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

fn read_load_1() -> Option<f32> {
    let text = fs::read_to_string("/proc/loadavg").ok()?;
    let first = text.split_whitespace().next()?;
    first.parse().ok()
}

fn read_meminfo() -> (u64, u64) {
    let Ok(text) = fs::read_to_string("/proc/meminfo") else {
        return (0, 0);
    };
    let mut total = 0u64;
    let mut avail = 0u64;
    let mut free = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;

    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next().unwrap_or("");
        let val: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        match key {
            "MemTotal:" => total = val,
            "MemAvailable:" => avail = val,
            "MemFree:" => free = val,
            "Buffers:" => buffers = val,
            "Cached:" => cached = val,
            _ => {}
        }
    }
    if avail == 0 && total > 0 {
        // Older kernels without MemAvailable
        avail = free.saturating_add(buffers).saturating_add(cached);
    }
    (total, avail)
}

fn read_battery_pct() -> Option<u8> {
    // Common Linux sysfs capacity path (first battery).
    for path in [
        "/sys/class/power_supply/BAT0/capacity",
        "/sys/class/power_supply/BAT1/capacity",
    ] {
        if let Ok(s) = fs::read_to_string(path) {
            if let Ok(v) = s.trim().parse::<u8>() {
                return Some(v.min(100));
            }
        }
    }
    None
}

fn thermal_hint(
    load_1: Option<f32>,
    mem_total_kb: u64,
    mem_avail_kb: u64,
    battery_pct: Option<u8>,
) -> String {
    let mut hints = Vec::new();
    if let Some(l) = load_1 {
        if l >= 8.0 {
            hints.push("hot-load");
        } else if l >= 4.0 {
            hints.push("warm-load");
        }
    }
    if mem_total_kb > 0 {
        let ratio = mem_avail_kb as f32 / mem_total_kb as f32;
        if ratio < 0.10 {
            hints.push("mem-critical");
        } else if ratio < 0.20 {
            hints.push("mem-tight");
        }
    }
    if let Some(b) = battery_pct {
        if b <= 15 {
            hints.push("battery-low");
        }
    }
    if hints.is_empty() {
        "nominal".into()
    } else {
        hints.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_snap(
        cpus: u32,
        load_1: Option<f32>,
        mem_total_kb: u64,
        mem_avail_kb: u64,
        battery_pct: Option<u8>,
    ) -> HostSnapshot {
        HostSnapshot {
            hostname: "test-host".into(),
            os: "linux".into(),
            arch: "x86_64".into(),
            cpus,
            load_1,
            mem_total_kb,
            mem_avail_kb,
            battery_pct,
            thermal_hint: "test".into(),
            entropy_source: "/dev/urandom".into(),
            probed_at: Utc::now(),
        }
    }

    #[test]
    fn policy_low_memory_reduces_parallel() {
        let snap = fake_snap(8, Some(0.2), 16_000_000, 800_000, None); // ~5% free
        let p = policy_from_snapshot(&snap);
        assert_eq!(p.max_tool_parallel, 1);
        assert!(p.notes.contains("low free memory") || p.notes.contains("memory"));
        assert!(p.max_agent_steps <= 16);
    }

    #[test]
    fn policy_high_load_lowers_effort() {
        let snap = fake_snap(2, Some(6.0), 16_000_000, 8_000_000, None); // load/cpu = 3
        let p = policy_from_snapshot(&snap);
        assert_eq!(p.preferred_effort, "low");
        assert!(p.notes.contains("high load"));
    }

    #[test]
    fn policy_healthy_host_defaults() {
        let snap = fake_snap(4, Some(0.3), 16_000_000, 10_000_000, None);
        let p = policy_from_snapshot(&snap);
        assert!(p.max_tool_parallel >= 2);
        assert_eq!(p.preferred_effort, "high");
        assert_eq!(p.max_agent_steps, 48);
    }

    #[test]
    fn policy_low_battery_is_conservative() {
        let snap = fake_snap(8, Some(0.1), 16_000_000, 10_000_000, Some(10));
        let p = policy_from_snapshot(&snap);
        assert_eq!(p.max_tool_parallel, 1);
        assert!(p.max_agent_steps <= 16);
        assert!(p.notes.contains("battery"));
    }

    #[test]
    fn probe_host_does_not_panic() {
        let snap = probe_host();
        assert!(!snap.hostname.is_empty());
        assert!(!snap.os.is_empty());
        assert!(!snap.arch.is_empty());
        assert!(snap.cpus >= 1);
        assert!(!snap.entropy_source.is_empty());
        assert!(!snap.thermal_hint.is_empty());
        // Round-trip through policy + format
        let _ = policy_from_snapshot(&snap);
        let text = format_probe(&snap);
        assert!(text.contains("aegis-hardware probe"));
        assert!(text.contains("throttle"));
    }

    #[test]
    fn apply_policy_overlays_object() {
        let snap = fake_snap(4, Some(0.5), 8_000_000, 4_000_000, None);
        let p = policy_from_snapshot(&snap);
        let out = apply_policy_to_json(json!({"model": "grok"}), &p);
        assert_eq!(out["model"], "grok");
        assert_eq!(out["throttle"]["preferred_effort"], p.preferred_effort);
        assert_eq!(out["throttle"]["max_tool_parallel"], p.max_tool_parallel);
        assert_eq!(out["throttle"]["max_agent_steps"], p.max_agent_steps);
    }

    #[test]
    fn apply_policy_wraps_non_object() {
        let p = ThrottlePolicy {
            max_tool_parallel: 2,
            preferred_effort: "medium".into(),
            max_agent_steps: 24,
            notes: "test".into(),
        };
        let out = apply_policy_to_json(json!(42), &p);
        assert_eq!(out["value"], 42);
        assert_eq!(out["throttle"]["max_tool_parallel"], 2);
    }
}
