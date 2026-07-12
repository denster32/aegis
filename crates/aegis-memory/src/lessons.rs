use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub ts: String,
    /// convention | command | fix | architecture | other
    pub kind: String,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_conf")]
    pub confidence: f32,
    #[serde(default = "default_hits")]
    pub hits: u32,
}

fn default_conf() -> f32 {
    0.5
}
fn default_hits() -> u32 {
    1
}

impl Lesson {
    pub fn score(&self) -> f32 {
        self.confidence * (1.0 + (self.hits as f32).ln_1p())
    }
}

/// Rank lessons and take top k.
pub fn top_lessons(mut lessons: Vec<Lesson>, k: usize) -> Vec<Lesson> {
    lessons.sort_by(|a, b| {
        b.score()
            .partial_cmp(&a.score())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lessons.truncate(k);
    lessons
}
