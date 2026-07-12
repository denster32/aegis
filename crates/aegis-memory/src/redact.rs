use regex::Regex;
use std::sync::OnceLock;

/// Strip obvious secrets before writing project memory.
pub fn redact_secrets(s: &str) -> String {
    static RES: OnceLock<Vec<Regex>> = OnceLock::new();
    let res = RES.get_or_init(|| {
        vec![
            Regex::new(r"(?i)(api[_-]?key|token|secret|password|authorization)\s*[:=]\s*\S+").unwrap(),
            Regex::new(r"Bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
            Regex::new(r"\bxai-[A-Za-z0-9]{20,}\b").unwrap(),
            Regex::new(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b").unwrap(),
            Regex::new(r"\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").unwrap(),
        ]
    });
    let mut out = s.to_string();
    for re in res {
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer() {
        let s = redact_secrets("Authorization: Bearer abc.def.ghi_extra_padding_here");
        assert!(s.contains("[REDACTED]"));
    }
}
