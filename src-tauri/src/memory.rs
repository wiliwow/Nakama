use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySummary {
    pub total_episodes: usize,
    pub total_facts: usize,
    pub total_goals: usize,
    pub total_failures: usize,
    pub namespaces: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryBuffer {
    pub passages: Vec<MemoryPassage>,
    pub facts: Vec<String>,
    pub token_budget: Option<usize>,
}

impl MemoryBuffer {
    pub fn from_recall(
        mut passages: Vec<MemoryPassage>,
        facts: Vec<String>,
        token_budget: Option<usize>,
    ) -> Self {
        passages.sort_by(|a, b| b.score.unwrap_or(0.0).partial_cmp(&a.score.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
        Self { passages, facts, token_budget }
    }

    pub fn is_empty(&self) -> bool {
        self.passages.is_empty() && self.facts.is_empty()
    }

    pub fn to_prompt(&self) -> String {
        let mut buf = String::new();
        if !self.facts.is_empty() {
            buf.push_str("## Known Facts\n");
            for f in &self.facts {
                buf.push_str("- ");
                buf.push_str(f);
                buf.push('\n');
            }
            buf.push('\n');
        }
        if !self.passages.is_empty() {
            buf.push_str("## Relevant Memories\n");
            for p in &self.passages {
                if let Some(score) = p.score {
                    buf.push_str(&format!("- [{}]\n{}\n\n", p.namespace.as_deref().unwrap_or("unknown"), p.content));
                }
            }
        }
        buf
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryPassage {
    pub id: Option<i64>,
    pub content: String,
    pub score: Option<f64>,
    pub namespace: Option<String>,
    pub created_at: Option<i64>,
}

impl From<crate::models::StashMemory> for MemoryPassage {
    fn from(m: crate::models::StashMemory) -> Self {
        Self {
            id: m.id.as_ref().and_then(|s| s.parse::<i64>().ok()),
            content: m.content,
            score: m.score,
            namespace: m.namespace,
            created_at: m.created_at.as_ref().and_then(|s| s.parse::<i64>().ok()),
        }
    }
}
