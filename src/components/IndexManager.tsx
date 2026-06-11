import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { useToast } from "../contexts/ToastContext";
import { useFocusTrap } from "../hooks/useFocusTrap";
import { IndexStatus, HealthStatus } from "../types";

const IndexManager: React.FC = () => {
  const { showToast } = useToast();
  const [isOpen, setIsOpen] = useState(false);
  const [indexStatus, setIndexStatus] = useState<IndexStatus | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [swiftideAvailable, setSwiftideAvailable] = useState<boolean | null>(
    null
  );
  useFocusTrap(isOpen);

  // Check if RAG is available on mount.
  const checkRag = async () => {
      try {
        const health = await invoke<HealthStatus>("rag_health_check");
        setHealthStatus(health);
        setSwiftideAvailable(health.llm_ok);
        const stats = await invoke<IndexStatus>("rag_index_stats");
        setIndexStatus(stats);
      } catch (err) {
        const msg = String(err);
        if (msg.includes("not found") || msg.includes("not enabled")) {
          setSwiftideAvailable(false);
        } else {
          console.error("[IndexManager] RAG initialisation failed:", err);
          setSwiftideAvailable(false);
        }
      }
    };
    checkRag();

  const refreshStatus = useCallback(async () => {
    if (!swiftideAvailable) {
      setSwiftideAvailable(false);
      return;
    }
    try {
      const health = await invoke<HealthStatus>("rag_health_check");
      setHealthStatus(health);
      const stats = await invoke<IndexStatus>("rag_index_stats");
      setIndexStatus(stats);
    } catch (err) {
      console.error("Failed to fetch status:", err);
      showToast(`Failed to refresh: ${err}`, { type: "error" });
      setSwiftideAvailable(false);
    }
  }, [swiftideAvailable, showToast]);

  const handleFileUpload = useCallback(async () => {
    if (!swiftideAvailable) {
      showToast(
        "RAG indexing not available in this build (swiftide integration disabled)",
        { type: "warning" }
      );
      return;
    }
    try {
      setLoading(true);

      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Text Files",
            extensions: ["txt", "md", "pdf", "json", "csv"],
          },
        ],
      });

      if (!selected) return;

      const selectedPaths = Array.isArray(selected) ? selected : [selected];
      let indexedCount = 0;
      let errorCount = 0;

      for (const filePath of selectedPaths) {
        try {
          const content = await readTextFile(filePath);
          const filename = filePath.split("/").pop() || filePath;

          await invoke<string>("rag_add_file", {
            filename,
            content,
          });
          indexedCount++;
          console.log(`Indexed file: ${filename}`);
        } catch (err) {
          console.error(`Failed to index file: ${filePath}`, err);
          errorCount++;
        }
      }

      if (indexedCount > 0) {
        showToast(`✓ Indexed ${indexedCount} file(s)`, {
          type: "success",
          duration: 3000,
        });
      }
      if (errorCount > 0) {
        showToast(`⚠ ${errorCount} file(s) failed to index`, {
          type: "error",
          duration: 3000,
        });
      }

      // Refresh status after indexing
      setTimeout(refreshStatus, 1000);
    } catch (err) {
      console.error("File upload error:", err);
      showToast(`File upload error: ${err}`, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [swiftideAvailable, refreshStatus, showToast]);

  const handleClearIndex = useCallback(async () => {
    if (!swiftideAvailable) {
      showToast(
        "RAG indexing not available in this build (swiftide integration disabled)",
        { type: "warning" }
      );
      return;
    }

    const confirmed = window.confirm(
      "Clear all indexed data? This cannot be undone."
    );
    if (!confirmed) return;

    try {
      setLoading(true);
      const result = await invoke<number>("rag_clear_index");
      setIndexStatus(null);
      showToast(`✓ Cleared ${result} document(s) from index`, {
        type: "success",
        duration: 3000,
      });
      setTimeout(refreshStatus, 500);
    } catch (err) {
      console.error("Clear index error:", err);
      showToast(`Error clearing index: ${err}`, { type: "error" });
    } finally {
      setLoading(false);
    }
  }, [swiftideAvailable, refreshStatus, showToast]);

  return (
    <>
      {/* Toggle Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="fixed top-4 right-4 z-40 px-4 py-2 rounded-lg bg-gradient-to-r from-blue-600 to-blue-800 text-white text-sm font-semibold hover:shadow-lg transition-all focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-950"
        title="Memory index management"
        aria-label="Toggle memory index manager"
      >
        📚 Index
      </button>

      {/* Modal */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div
            className="bg-[#181e36] border border-blue-900 rounded-2xl shadow-2xl p-6 w-full max-w-md max-h-[80vh] overflow-y-auto"
            role="dialog"
            aria-modal="true"
            aria-label="Memory Index Manager"
          >
            {/* Header */}
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-bold text-blue-100">
                Memory Index Manager
              </h2>
              <button
                onClick={() => setIsOpen(false)}
                className="text-gray-400 hover:text-white text-2xl focus:outline-none"
                aria-label="Close"
              >
                ✕
              </button>
            </div>

            {/* Health Status */}
            {swiftideAvailable === false ? (
              <div className="mb-6 p-4 bg-yellow-900/30 rounded-lg border border-yellow-800">
                <h3 className="text-sm font-semibold text-yellow-300 mb-2">
                  RAG Not Available
                </h3>
                <p className="text-xs text-gray-300">
                  RAG (Retrieval-Augmented Generation) indexing is not enabled in
                  this build. To enable, clone the swiftide repository and
                  enable the swiftide_integration feature.
                </p>
              </div>
            ) : (
              <div className="mb-6 p-4 bg-blue-900/30 rounded-lg border border-blue-800">
                <h3 className="text-sm font-semibold text-blue-300 mb-3">
                  System Health
                </h3>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center gap-2">
                    <span
                      className={
                        healthStatus?.embeddings_ok
                          ? "text-green-400"
                          : "text-red-400"
                      }
                    >
                      {healthStatus?.embeddings_ok ? "✓" : "✗"}
                    </span>
                    <span className="text-gray-300">
                      Embedding Engine
                      {healthStatus?.embeddings_error && (
                        <span className="text-red-300 text-xs block ml-6">
                          {healthStatus.embeddings_error}
                        </span>
                      )}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span
                      className={
                        healthStatus?.llm_ok ? "text-green-400" : "text-red-400"
                      }
                    >
                      {healthStatus?.llm_ok ? "✓" : "✗"}
                    </span>
                    <span className="text-gray-300">
                      LLM Backend (Ollama)
                      {healthStatus?.llm_error && (
                        <span className="text-red-300 text-xs block ml-6">
                          {healthStatus.llm_error}
                        </span>
                      )}
                    </span>
                  </div>
                </div>
                <button
                  onClick={refreshStatus}
                  disabled={loading}
                  className="mt-3 w-full text-xs py-1 px-2 rounded bg-blue-700 hover:bg-blue-600 text-blue-100 disabled:opacity-50"
                >
                  {loading ? "Refreshing..." : "Refresh"}
                </button>
              </div>
            )}

            {/* Index Statistics */}
            {indexStatus && (
              <div className="mb-6 p-4 bg-blue-900/20 rounded-lg border border-blue-800">
                <h3 className="text-sm font-semibold text-blue-300 mb-3">
                  Index Statistics
                </h3>
                <div className="space-y-2 text-xs text-gray-300">
                  <div className="flex justify-between">
                    <span>Indexed Documents:</span>
                    <span className="font-mono">
                      {indexStatus.indexed_documents}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Chunks:</span>
                    <span className="font-mono">{indexStatus.indexed_chunks}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Index Size:</span>
                    <span className="font-mono">
                      {indexStatus.estimated_index_size_mb} MB
                    </span>
                  </div>
                  {indexStatus.last_indexed_at && (
                    <div className="flex justify-between">
                      <span>Last Updated:</span>
                      <span className="font-mono">
                        {new Date(
                          indexStatus.last_indexed_at * 1000
                        ).toLocaleDateString()}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Actions */}
            {swiftideAvailable && (
              <div className="space-y-3">
                <button
                  onClick={handleFileUpload}
                  disabled={loading}
                  className="w-full py-2 px-3 rounded-lg bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 text-white font-semibold text-sm disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                >
                  {loading ? "Indexing..." : "📤 Upload & Index Files"}
                </button>

                <button
                  onClick={handleClearIndex}
                  disabled={loading}
                  className="w-full py-2 px-3 rounded-lg bg-red-900/40 hover:bg-red-900/60 text-red-300 font-semibold text-sm border border-red-800 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                >
                  🗑️ Clear Index
                </button>
              </div>
            )}

            <button
              onClick={() => setIsOpen(false)}
              className="w-full py-2 px-3 rounded-lg bg-gray-700 hover:bg-gray-600 text-gray-100 font-semibold text-sm transition-all mt-4"
            >
              Close
            </button>

            <p className="mt-4 text-xs text-gray-400 text-center">
              All indexed data is stored locally only. No uploads to cloud.
            </p>
          </div>
        </div>
      )}
    </>
  );
};

export default IndexManager;