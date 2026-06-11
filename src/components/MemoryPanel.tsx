import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../contexts/ToastContext";
import { useFocusTrap } from "../hooks/useFocusTrap";
import {
  MemoryItem,
  FactItem,
  GoalItem,
} from "../types";

interface MemoryPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

const MemoryPanel: React.FC<MemoryPanelProps> = ({ isOpen, onClose }) => {
  const { showToast } = useToast();
  const [localMemoryAvailable, setLocalMemoryAvailable] = useState<boolean | null>(null);
  const [summary, setSummary] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const [activeTab, setActiveTab] = useState<
    "memories" | "facts" | "goals" | "status"
  >("memories");
  const [searchQuery, setSearchQuery] = useState("");
  const [memories, setMemories] = useState<MemoryItem[]>([]);
  const [facts, setFacts] = useState<FactItem[]>([]);
  const [goals, setGoals] = useState<GoalItem[]>([]);
  const [newMemory, setNewMemory] = useState("");
  const [newGoal, setNewGoal] = useState("");

  const checkStatus = useCallback(async () => {
    setLoading(true);
    try {
      const s = await invoke<any>("memory_summary");
      setSummary(s);
      setLocalMemoryAvailable(true);
    } catch (err) {
      console.error("[Memory] memory_summary failed:", err);
      setLocalMemoryAvailable(false);
      setSummary(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      checkStatus();
    }
  }, [isOpen, checkStatus]);

  useEffect(() => {
    if (!isOpen) return;
    if (activeTab === "facts") {
      loadFacts();
    } else if (activeTab === "goals") {
      loadGoals();
    }
  }, [activeTab, isOpen]);

  useEffect(() => {
    if (activeTab === "memories" && searchQuery.length > 2) {
      const timer = setTimeout(handleSearch, 300);
      return () => clearTimeout(timer);
    }
  }, [searchQuery, activeTab]);

  const handleSearch = useCallback(async () => {
    if (!searchQuery.trim()) return;
    setLoading(true);
    try {
      const results = await invoke<MemoryItem[]>("memory_recall", {
        query: searchQuery,
        limit: 20,
      });
      setMemories(results);
    } catch (err) {
      const msg = `Search failed: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [searchQuery, showToast]);

  const loadFacts = useCallback(async () => {
    setLoading(true);
    try {
      const results = await invoke<FactItem[]>("memory_query_facts");
      setFacts(results);
    } catch (err) {
      const msg = `Failed to load facts: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  const loadGoals = useCallback(async () => {
    setLoading(true);
    try {
      const results = await invoke<GoalItem[]>("memory_list_goals");
      setGoals(results);
    } catch (err) {
      const msg = `Failed to load goals: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  const handleAddMemory = useCallback(async () => {
    if (!newMemory.trim()) return;
    setLoading(true);
    try {
      await invoke("memory_remember", {
        namespace: "nakama:conversations",
        content: newMemory,
      });
      setNewMemory("");
      showToast("Memory added", { type: "success", duration: 2000 });
      if (searchQuery.trim()) {
        handleSearch();
      }
    } catch (err) {
      const msg = `Failed to add memory: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [newMemory, searchQuery, handleSearch, showToast]);

  const handleAddGoal = useCallback(async () => {
    if (!newGoal.trim()) return;
    setLoading(true);
    try {
      await invoke("memory_create_goal", {
        namespace: "nakama:goals",
        description: newGoal,
      });
      setNewGoal("");
      loadGoals();
      showToast("Goal added", { type: "success", duration: 2000 });
    } catch (err) {
      const msg = `Failed to create goal: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [newGoal, loadGoals, showToast]);

  const handleCompleteGoal = useCallback(
    async (goalId?: string) => {
      if (!goalId) return;
      setLoading(true);
      try {
        await invoke("memory_complete_goal", {
          namespace: "nakama:goals",
          goal_id: goalId,
        });
        loadGoals();
        showToast("Goal marked complete", { type: "success" });
      } catch (err) {
        const msg = `Failed to complete goal: ${err}`;
        console.error(msg);
        showToast(msg, { type: "error" });
      } finally {
        setLoading(false);
      }
    },
    [loadGoals, showToast]
  );

  const handleConsolidate = useCallback(async () => {
    setLoading(true);
    try {
      await invoke("memory_consolidate");
      checkStatus();
      loadFacts();
      showToast("Consolidation complete", { type: "success" });
    } catch (err) {
      const msg = `Failed to consolidate: ${err}`;
      console.error(msg);
      showToast(msg, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [checkStatus, loadFacts, showToast]);

  const { releaseFocus: releaseMemoryFocus } = useFocusTrap(isOpen);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) releaseMemoryFocus();
      }}
    >
      <div
        className="bg-[#181e36] border border-blue-900 rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col"
        role="dialog"
        aria-modal="true"
        aria-label="Local Memory System"
      >
        <div className="flex justify-between items-center p-4 border-b border-slate-700/50">
          <h2 className="text-lg font-bold text-blue-100 flex items-center gap-2">
            <span className="text-xl">🧠</span>
            Local Memory
            {localMemoryAvailable !== null && (
              <span
                className={`w-2 h-2 rounded-full ${
                  localMemoryAvailable ? "bg-green-400" : "bg-red-400"
                }`}
                title={localMemoryAvailable ? "Available" : "Unavailable"}
              />
            )}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white text-xl"
            aria-label="Close memory panel"
          >
            ✕
          </button>
        </div>

        {!localMemoryAvailable && (
          <div className="mx-4 mt-3 p-3 bg-yellow-900/30 rounded-lg border border-yellow-800">
            <p className="text-xs text-yellow-300 text-center">
              Local memory not available. It runs entirely offline using SQLite.
              <button
                onClick={checkStatus}
                className="underline hover:text-yellow-100 ml-1"
              >
                Refresh
              </button>
              .
            </p>
          </div>
        )}

        {localMemoryAvailable && summary && (
          <div className="mx-4 mt-3 grid grid-cols-4 gap-2">
            <div className="p-2 bg-blue-900/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-blue-200">
                {summary.total_episodes}
              </div>
              <div className="text-xs text-gray-400">Episodes</div>
            </div>
            <div className="p-2 bg-green-900/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-green-200">
                {summary.total_facts}
              </div>
              <div className="text-xs text-gray-400">Facts</div>
            </div>
            <div className="p-2 bg-purple-900/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-purple-200">
                {summary.total_goals}
              </div>
              <div className="text-xs text-gray-400">Goals</div>
            </div>
            <div className="p-2 bg-orange-900/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-orange-200">
                {summary.namespaces.length}
              </div>
              <div className="text-xs text-gray-400">Namespaces</div>
            </div>
          </div>
        )}

        <div className="flex border-b border-slate-700/50 px-4 mt-3">
          {[
            { key: "memories", label: "🔍 Memories" },
            { key: "facts", label: "📋 Facts" },
            { key: "goals", label: "🎯 Goals" },
            { key: "status", label: "⚙️ Status" },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key as any)}
              className={`px-4 py-2 text-sm font-semibold border-b-2 transition-all ${
                activeTab === tab.key
                  ? "text-blue-300 border-blue-500"
                  : "text-gray-400 border-transparent hover:text-gray-300"
              }`}
            >
              {tab.label}
            </button>
          ))}
          <button
            onClick={handleConsolidate}
            disabled={loading || !localMemoryAvailable}
            className="ml-auto px-3 py-1 text-xs bg-blue-700 hover:bg-blue-600 text-blue-100 rounded disabled:opacity-50"
          >
            Consolidate
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {activeTab === "memories" && (
            <div className="space-y-3">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search memories..."
                  className="flex-1 bg-[#232a4a] text-white px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  aria-label="Search memories"
                />
                <button
                  onClick={handleSearch}
                  disabled={!searchQuery.trim() || !localMemoryAvailable}
                  className="px-3 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50"
                >
                  Search
                </button>
              </div>

              <div className="flex gap-2">
                <input
                  type="text"
                  value={newMemory}
                  onChange={(e) => setNewMemory(e.target.value)}
                  placeholder="Add a memory observation..."
                  className="flex-1 bg-[#232a4a] text-white px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  aria-label="New memory observation"
                />
                <button
                  onClick={handleAddMemory}
                  disabled={!newMemory.trim() || !localMemoryAvailable}
                  className="px-3 py-2 bg-green-600 hover:bg-green-500 text-white text-sm rounded-lg disabled:opacity-50"
                >
                  Remember
                </button>
              </div>

              {loading ? (
                <div className="text-center py-8 text-gray-400 animate-pulse">
                  Loading memories...
                </div>
              ) : memories.length > 0 ? (
                <div className="space-y-2">
                  {memories.map((mem, idx) => (
                    <div
                      key={mem.id ?? idx}
                      className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50"
                    >
                      <p className="text-sm text-slate-200">{mem.content}</p>
                      <div className="flex justify-between items-center mt-1 text-xs text-gray-500">
                        <span>
                          {mem.namespace} •{" "}
                          {mem.score
                            ? `${(mem.score * 100).toFixed(1)}% match`
                            : ""}
                        </span>
                        <span>{mem.created_at || ""}</span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500 text-sm">
                  {searchQuery
                    ? "No matching memories found"
                    : "Search memories or add new observations above"}
                </div>
              )}
            </div>
          )}

          {activeTab === "facts" && (
            <div className="space-y-3">
              {loading ? (
                <div className="text-center py-8 text-gray-400 animate-pulse">
                  Loading facts...
                </div>
              ) : facts.length > 0 ? (
                <div className="space-y-2">
                  {facts.map((fact, idx) => (
                    <div
                      key={fact.id ?? idx}
                      className="p-3 bg-emerald-900/20 rounded-lg border border-emerald-800/50"
                    >
                      <p className="text-sm text-emerald-200">{fact.fact}</p>
                      <div className="flex justify-between items-center mt-1 text-xs text-gray-500">
                        <span>
                          Confidence:{" "}
                          {fact.confidence != null
                            ? `${(fact.confidence * 100).toFixed(0)}%`
                            : "N/A"}
                        </span>
                        <span>{fact.namespace}</span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500 text-sm">
                  No facts yet. Run consolidation to extract facts from memories.
                </div>
              )}
            </div>
          )}

          {activeTab === "goals" && (
            <div className="space-y-3">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newGoal}
                  onChange={(e) => setNewGoal(e.target.value)}
                  placeholder="Add a new goal..."
                  className="flex-1 bg-[#232a4a] text-white px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  aria-label="New goal description"
                />
                <button
                  onClick={handleAddGoal}
                  disabled={!newGoal.trim() || !localMemoryAvailable}
                  className="px-3 py-2 bg-amber-600 hover:bg-amber-500 text-white text-sm rounded-lg disabled:opacity-50"
                >
                  Add Goal
                </button>
              </div>

              {loading ? (
                <div className="text-center py-8 text-gray-400 animate-pulse">
                  Loading goals...
                </div>
              ) : goals.length > 0 ? (
                <div className="space-y-2">
                  {goals.map((goal, idx) => (
                    <div
                      key={goal.id ?? idx}
                      className="p-3 bg-slate-800/50 rounded-lg border border-slate-700/50"
                    >
                      <div className="flex justify-between items-start">
                        <p
                          className={`text-sm ${
                            goal.status === "completed"
                              ? "text-emerald-300 line-through"
                              : goal.status === "failed"
                                ? "text-red-300"
                                : "text-slate-200"
                          }`}
                        >
                          {goal.description}
                        </p>
                        <span
                          className={`text-xs font-semibold px-2 py-0.5 rounded ${
                            goal.status === "completed"
                              ? "bg-emerald-900/30 text-emerald-300"
                              : goal.status === "in_progress"
                                ? "bg-blue-900/30 text-blue-300"
                                : goal.status === "failed"
                                  ? "bg-red-900/30 text-red-300"
                                  : "bg-gray-700 text-gray-300"
                          }`}
                        >
                          {goal.status}
                        </span>
                      </div>
                      {goal.progress != null && goal.progress > 0 && (
                        <div className="w-full bg-slate-700 rounded-full h-2 mt-2">
                          <div
                            className="bg-blue-500 h-2 rounded-full transition-all"
                            style={{
                              width: `${Math.min(goal.progress, 100)}%`,
                            }}
                          />
                        </div>
                      )}
                      {goal.status !== "completed" && goal.status !== "failed" && (
                        <button
                          onClick={() => handleCompleteGoal(goal.id)}
                          disabled={loading}
                          className="mt-2 text-xs text-blue-400 hover:text-blue-300 transition-colors"
                        >
                          Mark Complete
                        </button>
                      )}
                      <div className="text-xs text-gray-500 mt-1">
                        {goal.namespace}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500 text-sm">
                  No goals yet. Add goals to track agent objectives.
                </div>
              )}
            </div>
          )}

          {activeTab === "status" && (
            <div className="space-y-4">
              <div className="p-4 bg-blue-900/30 rounded-lg border border-blue-800/50">
                <h3 className="text-sm font-semibold text-blue-300 mb-3">
                  Memory Status
                </h3>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-300">Backend:</span>
                    <span className="text-green-400 font-semibold">
                      SQLite (local)
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-300">Status:</span>
                    <span
                      className={
                        localMemoryAvailable
                          ? "text-green-400 font-semibold"
                          : "text-red-400 font-semibold"
                      }
                    >
                      {localMemoryAvailable ? "Available" : "Unavailable"}
                    </span>
                  </div>
                </div>
                <button
                  onClick={checkStatus}
                  disabled={loading}
                  className="mt-3 w-full text-xs py-1 px-2 rounded bg-blue-700 hover:bg-blue-600 text-blue-100 disabled:opacity-50"
                >
                  {loading ? "Refreshing..." : "Refresh"}
                </button>
              </div>

              {summary && (
                <div className="p-4 bg-purple-900/20 rounded-lg border border-purple-800/50">
                  <h3 className="text-sm font-semibold text-purple-300 mb-3">
                    Statistics
                  </h3>
                  <div className="space-y-2 text-sm text-gray-300">
                    <div className="flex justify-between">
                      <span>Total Episodes:</span>
                      <span className="font-mono">
                        {summary.total_episodes}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Total Facts:</span>
                      <span className="font-mono">
                        {summary.total_facts}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Total Goals:</span>
                      <span className="font-mono">
                        {summary.total_goals}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Namespaces:</span>
                      <span className="font-mono text-xs">
                        {summary.namespaces.join(", ") || "—"}
                      </span>
                    </div>
                  </div>
                </div>
              )}

              <div className="p-4 bg-slate-900/50 rounded-lg border border-slate-700/50">
                <h3 className="text-sm font-semibold text-slate-300 mb-3">
                  Setup
                </h3>
                <p className="text-xs text-gray-400">
                  No external services required. Memory is stored locally in SQLite.
                  To configure an external LLM endpoint, set the
                  <code className="text-blue-300"> LLM_ENDPOINT </code> and
                  <code className="text-blue-300"> LLM_MODEL </code> environment variables.
                </p>
              </div>
            </div>
          )}
        </div>

        <div className="p-3 border-t border-slate-700/50 flex justify-between items-center">
          <p className="text-xs text-gray-500">
            Local memory for your AI agent — works fully offline
          </p>
          <button
            onClick={onClose}
            className="px-4 py-1.5 bg-slate-700 hover:bg-slate-600 text-sm text-gray-100 rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default MemoryPanel;
