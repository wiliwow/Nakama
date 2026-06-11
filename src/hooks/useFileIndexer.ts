import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface FileToIndex {
  name: string;
  content: string;
}

interface IndexingError {
  file: string;
  error: string;
}

export function useFileIndexer() {
  const [indexing, setIndexing] = useState(false);
  const [indexedCount, setIndexedCount] = useState(0);
  const [errors, setErrors] = useState<IndexingError[]>([]);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const indexFiles = useCallback(
    async (files: FileToIndex[]): Promise<{ success: boolean; count: number; errors: IndexingError[] }> => {
      if (files.length === 0) {
        return { success: false, count: 0, errors: [] };
      }

      setIndexing(true);
      setErrors([]);
      setMessage(null);

      let successCount = 0;
      const newErrors: IndexingError[] = [];

      for (const file of files) {
        try {
          const result = await invoke<string>("rag_add_file", {
            filename: file.name,
            content: file.content,
          });
          console.log(`[RAG] Indexed: ${result}`);
          successCount++;
        } catch (err) {
          if (String(err).includes("swiftide integration not enabled")) {
            newErrors.push({
              file: file.name,
              error: "RAG not enabled in this build",
            });
          } else {
            newErrors.push({
              file: file.name,
              error: String(err),
            });
          }
        }
      }

      setIndexedCount((prev) => prev + successCount);
      setErrors(newErrors);

      if (successCount === files.length && newErrors.length === 0) {
        setMessage({
          type: "success",
          text: `✓ Indexed ${successCount}/${files.length} file(s)`,
        });
      } else if (successCount > 0) {
        setMessage({
          type: "error",
          text: `Indexed ${successCount}/${files.length}. ${newErrors.length} failed.`,
        });
      } else {
        setMessage({
          type: "error",
          text: `⚠️ Failed to index any files. ${newErrors.length} error(s).`,
        });
      }

      setIndexing(false);
      return { success: successCount > 0, count: successCount, errors: newErrors };
    },
    []
  );

  const clearIndex = useCallback(async (): Promise<number | null> => {
    if (!confirm("Clear all indexed data? This cannot be undone.")) {
      return null;
    }

    setIndexing(true);
    try {
      const result = await invoke<number>("rag_clear_index");
      setIndexedCount(0);
      setErrors([]);
      setMessage({
        type: "success",
        text: `✓ Cleared ${result} document(s) from index`,
      });
      return result;
    } catch (err) {
      setMessage({
        type: "error",
        text: `Error clearing index: ${err}`,
      });
      return null;
    } finally {
      setIndexing(false);
    }
  }, []);

  return {
    indexFiles,
    clearIndex,
    indexing,
    indexedCount,
    errors,
    message,
  };
}