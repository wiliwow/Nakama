use crate::local_memory::LocalMemoryClient;
use crate::memory::{MemoryBuffer, MemoryPassage};
use crate::models::{MemorySummary, StashFact, StashGoal, StashMemory};
use tracing::warn;

pub struct MemoryAgent {
    client: LocalMemoryClient,
}

impl MemoryAgent {
    pub fn new(client: LocalMemoryClient) -> Self {
        Self { client }
    }

    pub fn new_in_memory() -> Self {
        Self { client: LocalMemoryClient::new_in_memory().expect("Failed to init in-memory local memory") }
    }

    pub fn remember_conversation_turn(&self, role: &str, content: &str) -> Result<i64, String> {
        let episode = format!("{}: {}", role, content);
        self.client.remember("nakama:conversations", &episode, None)
    }

    pub fn remember_action(&self, action_type: &str, result: &str) -> Result<i64, String> {
        let episode = format!("Action [{}]: {}", action_type, result);
        self.client.remember("nakama:actions", &episode, None)
    }

    pub fn remember(&self, namespace: &str, content: &str, metadata: Option<&serde_json::Value>) -> Result<i64, String> {
        self.client.remember(namespace, content, metadata.cloned())
    }

    pub fn remember_preference(&self, preference: &str, value: &str) -> Result<i64, String> {
        let episode = format!("Preference [{}]: {}", preference, value);
        self.client.remember("nakama:preferences", &episode, None)
    }

    pub fn remember_system_event(&self, event: &str, details: Option<&str>) -> Result<i64, String> {
        let content = match details { Some(d) => format!("{}: {}", event, d), None => event.to_string() };
        self.client.remember("nakama:system", &content, None)
    }

    pub fn recall_all(&self, query: &str, limit_per_ns: Option<usize>) -> Result<Vec<StashMemory>, String> {
        let namespaces = self.client.list_namespaces()?;
        let mut all_memories = Vec::new();
        for ns in &namespaces {
            let limit = limit_per_ns.unwrap_or(5);
            match self.client.recall(ns, query, Some(limit)) {
                Ok(mut memories) => {
                    for m in &mut memories { m.namespace = Some(ns.clone()); }
                    all_memories.extend(memories);
                }
                Err(e) => { warn!(error = %e, namespace = %ns, "memory: recall failed for namespace"); continue; }
            }
        }
        all_memories.sort_by(|a, b| { b.score.unwrap_or(0.0).partial_cmp(&a.score.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal) });
        Ok(all_memories)
    }

    pub fn recall_from(&self, namespace: &str, query: &str, limit: Option<usize>) -> Result<Vec<StashMemory>, String> {
        self.client.recall(namespace, query, limit)
    }

    pub fn query_facts_ns(&self, namespace: &str) -> Result<Vec<StashFact>, String> {
        self.client.query_facts(namespace)
    }

    pub fn get_facts(&self) -> Result<Vec<StashFact>, String> {
        let namespaces = self.client.list_namespaces()?;
        let mut all_facts = Vec::new();
        for ns in &namespaces {
            match self.client.query_facts(ns) {
                Ok(facts) => all_facts.extend(facts),
                Err(e) => warn!(error = %e, namespace = %ns, "memory: query_facts failed"),
            }
        }
        Ok(all_facts)
    }

    pub fn get_goals(&self) -> Result<Vec<StashGoal>, String> {
        let namespaces = self.client.list_namespaces()?;
        let mut all_goals = Vec::new();
        for ns in &namespaces {
            match self.client.list_goals(ns) {
                Ok(goals) => all_goals.extend(goals),
                Err(e) => warn!(error = %e, namespace = %ns, "memory: list_goals failed"),
            }
        }
        Ok(all_goals)
    }

    pub fn create_goal(&self, namespace: &str, description: &str) -> Result<i64, String> {
        self.client.create_goal(namespace, description)
    }

    pub fn list_goals_ns(&self, namespace: &str) -> Result<Vec<StashGoal>, String> {
        self.client.list_goals(namespace)
    }

    pub fn consolidate_ns(&self, namespace: &str) -> Result<(), String> {
        self.client.consolidate(namespace)
    }

    pub fn complete_goal(&self, namespace: &str, goal_id: i64) -> Result<(), String> {
        self.client.complete_goal(namespace, goal_id)
    }

    pub fn record_failure(&self, namespace: &str, what: &str, why: &str, lesson: &str) -> Result<(), String> {
        self.client.create_failure(namespace, what, why, lesson)
    }

    pub fn get_summary(&self) -> Result<MemorySummary, String> {
        self.client.get_summary()
    }

    pub fn build_memory_prompt(&self, user_prompt: &str) -> Result<String, String> {
        let memories = self.recall_all(user_prompt, Some(10))?;
        let facts = self.get_facts().unwrap_or_default();
        let buffer = MemoryBuffer::from_recall(
            memories.into_iter().map(MemoryPassage::from).collect(),
            facts.into_iter().map(|f| f.fact).collect(),
            None,
        );
        if buffer.is_empty() {
            return Ok(user_prompt.to_string());
        }
        Ok(format!("{}\n\n### CURRENT TASK\n{}", buffer.to_prompt(), user_prompt))
    }

    pub fn set_working_context(&self, task: &str, active_goals: &[String]) -> Result<(), String> {
        let ctx_json = serde_json::json!({ "current_task": task, "active_goals": active_goals });
        self.client.set_context("nakama:system", &ctx_json.to_string())
    }

    pub fn init_default_namespaces(&self) -> Result<(), String> {
        let namespaces = [
            "nakama:conversations",
            "nakama:actions",
            "nakama:preferences",
            "nakama:system",
            "nakama:failures",
            "nakama:goals",
        ];
        for ns in &namespaces {
            let _ = self.client.create_namespace(ns);
        }
        let _ = self.client.set_context("nakama:system", r#"{"current_task":"idle","active_goals":[]}"#);
        let _ = self.client.remember("nakama:system", "Agent initialized", None);
        Ok(())
    }
}
