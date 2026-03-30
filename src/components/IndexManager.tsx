import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";

interface IndexStatus {
  total_indexed_bytes: number;
  indexed_documents: number;
  indexed_chunks: number;
  last_indexed_at: number | null;
  estimated_index_size_mb: number;
}

interface HealthStatus {
  embeddings_ok: boolean;
  embeddings_error: string | null;
  llm_ok: boolean;
  llm_error: string | null;
}

const IndexManager: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [indexStatus] = useState<IndexStatus | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  // Load index status on open
  useEffect(() => {
    if (isOpen) {
      refreshStatus();
    }
  }, [isOpen]);

  const refreshStatus = async () => {
    try {
      // In a full implementation, you'd have backend commands to fetch these stats
      // For now, we'll show placeholder data with health check
      const health = await invoke<HealthStatus>("rag_health_check");
      setHealthStatus(health);
    } catch (err) {
      console.error("Failed to fetch status:", err);
      setMessage({ type: "error", text: `Error: ${String(err)}` });
    }
  };

  const handleFileUpload = async () => {
    try {
      setLoading(true);
      
      // Open file picker
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

      for (const filePath of selectedPaths) {
        try {
          const content = await readTextFile(filePath);
          const filename = filePath.split("/").pop() || filePath;
          
          // Invoke RAG to index the file
          const result = await invoke<string>("rag_add_file", {
            filename,
            content,
          });
          
          console.log("Indexed file:", result);
          setMessage({ type: "success", text: `✓ Indexed: ${filename}` });
        } catch (err) {
          console.error("Failed to index file:", err);
          setMessage({ type: "error", text: `Failed to index file: ${String(err)}` });
        }
      }

      // Refresh status after indexing
      setTimeout(refreshStatus, 1000);
    } catch (err) {
      console.error("File upload error:", err);
      setMessage({ type: "error", text: `Error: ${String(err)}` });
    } finally {
      setLoading(false);
    }
  };

  const handleClearIndex = async () => {
    if (!confirm("Clear all indexed data? This cannot be undone.")) {
      return;
    }

    try {
      setLoading(true);
      // Backend command to clear index (to be implemented in rag_indexer)
      // For now, we'll just show the intent
      setMessage({ type: "error", text: "Clear feature coming soon" });
    } catch (err) {
      console.error("Clear index error:", err);
      setMessage({ type: "error", text: `Error: ${String(err)}` });
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      {/* Toggle Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="fixed top-4 right-4 z-40 px-4 py-2 rounded-lg bg-gradient-to-r from-blue-600 to-blue-800 text-white text-sm font-semibold hover:shadow-lg transition-all"
        title="Memory index management"
      >
        📚 Index
      </button>

      {/* Modal */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="bg-[#181e36] border border-blue-900 rounded-2xl shadow-2xl p-6 w-full max-w-md max-h-[80vh] overflow-y-auto">
            {/* Header */}
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-bold text-blue-100">Memory Index Manager</h2>
              <button
                onClick={() => setIsOpen(false)}
                className="text-gray-400 hover:text-white text-2xl"
              >
                ✕
              </button>
            </div>

            {/* Health Status */}
            <div className="mb-6 p-4 bg-blue-900/30 rounded-lg border border-blue-800">
              <h3 className="text-sm font-semibold text-blue-300 mb-3">System Health</h3>
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-2">
                  <span className={healthStatus?.embeddings_ok ? "text-green-400" : "text-red-400"}>
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
                  <span className={healthStatus?.llm_ok ? "text-green-400" : "text-red-400"}>
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
                Refresh
              </button>
            </div>

            {/* Index Statistics */}
            {indexStatus && (
              <div className="mb-6 p-4 bg-blue-900/20 rounded-lg border border-blue-800">
                <h3 className="text-sm font-semibold text-blue-300 mb-3">Index Statistics</h3>
                <div className="space-y-2 text-xs text-gray-300">
                  <div className="flex justify-between">
                    <span>Indexed Documents:</span>
                    <span className="font-mono">{indexStatus.indexed_documents}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Chunks:</span>
                    <span className="font-mono">{indexStatus.indexed_chunks}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Index Size:</span>
                    <span className="font-mono">{indexStatus.estimated_index_size_mb} MB</span>
                  </div>
                  {indexStatus.last_indexed_at && (
                    <div className="flex justify-between">
                      <span>Last Updated:</span>
                      <span className="font-mono">
                        {new Date(indexStatus.last_indexed_at * 1000).toLocaleDateString()}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Messages */}
            {message && (
              <div
                className={`mb-4 p-3 rounded-lg text-sm ${
                  message.type === "success"
                    ? "bg-green-900/30 text-green-300 border border-green-800"
                    : "bg-red-900/30 text-red-300 border border-red-800"
                }`}
              >
                {message.text}
              </div>
            )}

            {/* Actions */}
            <div className="space-y-3">
              <button
                onClick={handleFileUpload}
                disabled={loading}
                className="w-full py-2 px-3 rounded-lg bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 text-white font-semibold text-sm disabled:opacity-50 transition-all"
              >
                {loading ? "Indexing..." : "📤 Upload & Index Files"}
              </button>

              <button
                onClick={handleClearIndex}
                disabled={loading}
                className="w-full py-2 px-3 rounded-lg bg-red-900/40 hover:bg-red-900/60 text-red-300 font-semibold text-sm border border-red-800 disabled:opacity-50 transition-all"
              >
                🗑️ Clear Index
              </button>

              <button
                onClick={() => setIsOpen(false)}
                className="w-full py-2 px-3 rounded-lg bg-gray-700 hover:bg-gray-600 text-gray-100 font-semibold text-sm transition-all"
              >
                Close
              </button>
            </div>

            {/* Info Text */}
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
