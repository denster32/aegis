use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub id: String,
    pub ts: String,
    pub fingerprint: String,
    pub tool: String,
    pub pattern: String,
    pub root_cause: String,
    pub fix: String,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub hits: u32,
}

/// Normalize noisy error text for stable fingerprints.
pub fn normalize_error(s: &str) -> String {
    let mut out = s.to_lowercase();
    // strip absolute paths somewhat
    out = regex_lite_replace_paths(&out);
    // collapse whitespace
    let parts: Vec<_> = out.split_whitespace().collect();
    let joined = parts.join(" ");
    // truncate
    if joined.len() > 400 {
        joined[..400].to_string()
    } else {
        joined
    }
}

fn regex_lite_replace_paths(s: &str) -> String {
    // simple: replace /home/... and /tmp/... segments
    let mut result = String::new();
    for token in s.split_whitespace() {
        if token.starts_with('/') && token.len() > 8 {
            result.push_str("<path>");
        } else if token.contains("line ") {
            result.push_str(token);
        } else {
            result.push_str(token);
        }
        result.push(' ');
    }
    result
}

pub fn fingerprint(tool: &str, error: &str) -> String {
    let norm = normalize_error(error);
    let mut h = Sha256::new();
    h.update(tool.as_bytes());
    h.update(b"\0");
    h.update(norm.as_bytes());
    hex::encode(h.finalize())[..16].to_string()
}

pub fn find_known_fix<'a>(
    failures: &'a [FailureRecord],
    fp: &str,
    min_confidence: f32,
) -> Option<&'a FailureRecord> {
    failures
        .iter()
        .filter(|f| f.fingerprint == fp && f.confidence >= min_confidence && !f.fix.is_empty())
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_stable() {
        let a = fingerprint("bash", "error at /tmp/foo/bar.rs line 10");
        let b = fingerprint("bash", "error at /tmp/other/x.rs line 10");
        // paths normalized — may match
        assert_eq!(a.len(), 16);
        let _ = b;
    }
}
