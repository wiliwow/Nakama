import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MemoryItem } from "../types";

interface UseMemoryRecallOptions {
  localMemoryAvailable: boolean;
  memoryCount: { episodes: number };
  namespace?: string;
}

interface MemoryRecallResult {
  text: string;
  formatted: string;
  relevant: boolean;
  memories: MemoryItem[];
}

export function useMemoryRecall({
  localMemoryAvailable,
  memoryCount,
  namespace,
}: UseMemoryRecallOptions) {
  const [recalling, setRecalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const recall = useCallback(
    async (query: string): Promise<MemoryRecallResult> => {
      if (!localMemoryAvailable || memoryCount.episodes === 0) {
        return { text: "", formatted: "", relevant: false, memories: [] };
      }

      setRecalling(true);
      setError(null);

      try {
        const memories = await invoke<MemoryItem[]>("memory_recall", {
          query,
          namespace,
          limit: 5,
        });

        if (memories && memories.length > 0) {
          const relevantMemories = memories.filter(
            (m) => (m.score ?? 0) > 0.3
          );

          const formatted = relevantMemories
            .map(
              (m) =>
                `- [${m.namespace || "memory"}] ${m.content} (relevance: ${((m.score ?? 0) * 100).toFixed(1)}%)`
            )
            .join("\n");

          const plainText = relevantMemories
            .map((m) => `${m.content}`)
            .join("\n");

          return {
            text: plainText,
            formatted,
            relevant: true,
            memories: relevantMemories,
          };
        }

        return { text: "", formatted: "", relevant: false, memories: [] };
      } catch (err) {
        const msg = `Memory recall failed: ${err}`;
        console.warn("[Memory]", msg);
        setError(msg);
        return { text: "", formatted: "", relevant: false, memories: [] };
      } finally {
        setRecalling(false);
      }
    },
    [localMemoryAvailable, memoryCount.episodes, namespace]
  );

  const remember = useCallback(
    async (role: string, content: string): Promise<boolean> => {
      if (!localMemoryAvailable) return false;

      try {
        await invoke("memory_remember", {
          namespace: namespace || "nakama:conversations",
          content: `${role}: ${content}`,
        });
        return true;
      } catch (err) {
        console.warn("[Memory] Remember failed:", err);
        setError(`Remember failed: ${err}`);
        return false;
      }
    },
    [localMemoryAvailable, namespace]
  );

  return { recall, remember, recalling, error };
}
