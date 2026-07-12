//! Tiny crate for dogfooding Aegis learning + missions.

pub fn greet(name: &str) -> String {
    format!("hello, {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets() {
        assert_eq!(greet("aegis"), "hello, aegis");
    }
}
