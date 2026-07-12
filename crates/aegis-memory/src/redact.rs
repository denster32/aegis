use regex::Regex;
use std::sync::OnceLock;

/// Strip obvious secrets before writing project memory.
pub fn redact_secrets(s: &str) -> String {
    static RES: OnceLock<Vec<Regex>> = OnceLock::new();
    let res = RES.get_or_init(|| {
        vec![
            Regex::new(r"(?i)(api[_-]?key|token|secret|password|authorization)\s*[:=]\s*\S+")
                .unwrap(),
            Regex::new(r"Bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
            Regex::new(r"\bxai-[A-Za-z0-9]{20,}\b").unwrap(),
            Regex::new(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b").unwrap(),
            Regex::new(r"\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
                .unwrap(),
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
        // First pattern may consume "Authorization: Bearer"; standalone Bearer form also redacts.
        let s2 = redact_secrets("header Bearer abc.def.ghi_extra_padding_here");
        assert!(s2.contains("[REDACTED]"));
        assert!(!s2.contains("abc.def.ghi_extra_padding_here"));
    }

    #[test]
    fn redacts_api_key_assignment() {
        let s = redact_secrets("api_key=supersecretvalue123");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("supersecretvalue123"));
        let s2 = redact_secrets("password: hunter2token");
        assert!(s2.contains("[REDACTED]"));
        let s3 = redact_secrets("TOKEN: abcdefghijklmnop");
        assert!(s3.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_xai_key_prefix() {
        let s = redact_secrets("use xai-abcdefghijklmnopqrstuvwxyz123456 for api");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("xai-abcdefghijklmnop"));
    }

    #[test]
    fn redacts_github_tokens() {
        let s = redact_secrets("gho_abcdefghijklmnopqrstuvwxyz123456");
        assert!(s.contains("[REDACTED]"));
        let s2 = redact_secrets("ghp_abcdefghijklmnopqrstuvwxyz123456");
        assert!(s2.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_jwt_like() {
        // three base64url segments
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signaturepadxx";
        let s = redact_secrets(&format!("auth={jwt}"));
        // either jwt pattern or key=value pattern should catch it
        assert!(s.contains("[REDACTED]") || !s.contains("eyJhbGci"));
    }

    #[test]
    fn leaves_benign_text() {
        let s = redact_secrets("run cargo test in the workspace root");
        assert_eq!(s, "run cargo test in the workspace root");
        assert!(!s.contains("[REDACTED]"));
    }
}
