use serde::{Deserialize, Serialize};

/// Represents a single observation or event stored in Stash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEpisode {
    pub namespace: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: Option<String>,
}

/// Represents a recalled memory item from Stash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashMemory {
    pub id: Option<String>,
    pub content: String,
    pub score: Option<f64>,
    pub namespace: Option<String>,
    pub created_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Represents a fact extracted by Stash consolidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashFact {
    pub id: Option<String>,
    pub fact: String,
    pub confidence: Option<f64>,
    pub namespace: Option<String>,
}

/// The overall state of the Stash memory system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemorySummary {
    pub total_episodes: usize,
    pub total_facts: usize,
    pub total_goals: usize,
    pub total_failures: usize,
    pub namespaces: Vec<String>,
}

/// A goal tracked by Stash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashGoal {
    pub id: Option<String>,
    pub description: String,
    pub status: String,
    pub progress: Option<f64>,
    pub namespace: Option<String>,
}

/// Structured working context for the agent's current task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContext {
    pub current_task: Option<String>,
    pub active_goals: Vec<String>,
    pub recent_observations: Vec<String>,
}