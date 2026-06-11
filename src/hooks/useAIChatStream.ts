import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ConversationMessage } from "../types";

interface UseAIChatStreamCallbacks {
  onChunk?: (chunk: string) => void;
  onComplete?: () => void;
  onError?: (error: string) => void;
}

export function useAIChatStream({
  onChunk,
  onComplete,
  onError,
}: UseAIChatStreamCallbacks) {
  const [streaming, setStreaming] = useState(false);
  const [streamedText, setStreamedText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const unsubscribesRef = useRef<{
    chunk?: () => void;
    error?: () => void;
    done?: () => void;
  }>({});

  const cancel = useCallback(() => {
    unsubscribesRef.current.chunk?.();
    unsubscribesRef.current.error?.();
    unsubscribesRef.current.done?.();
    unsubscribesRef.current = {};
    setStreaming(false);
  }, []);

  const sendMessage = useCallback(
    async (
      prompt: string,
      conversationHistory: ConversationMessage[]
    ): Promise<{ success: boolean; error?: string }> => {
      if (streaming) {
        return { success: false, error: "Already streaming" };
      }

      setStreaming(true);
      setStreamedText("");
      setError(null);

      try {
        const unlistenChunk = await listen<string>("ai-stream-chunk", (event) => {
          const chunk = event.payload ?? "";
          setStreamedText((prev) => prev + chunk);
          onChunk?.(chunk);
        });

        const unlistenError = await listen<string>("ai-stream-error", (event) => {
          const err = event.payload ?? "Unknown error";
          console.error("[AI] Stream error:", err);
          setError(err);
          onError?.(err);
          cancel();
        });

        const unlistenDone = await listen<void>("ai-stream-done", () => {
          setStreaming(false);
          onComplete?.();
          unlistenChunk();
          unlistenError();
          unsubscribesRef.current = {};
        });

        unsubscribesRef.current = {
          chunk: unlistenChunk,
          error: unlistenError,
          done: unlistenDone,
        };

        await invoke("execute_ai_with_actions", {
          prompt,
          conversationHistory,
        });

        return { success: true };
      } catch (err) {
        const msg = `AI execution failed: ${err}`;
        console.error("[AI]", msg);
        setError(msg);
        setStreaming(false);
        onError?.(msg);
        return { success: false, error: msg };
      }
    },
    [streaming, cancel, onChunk, onComplete, onError]
  );

  return {
    sendMessage,
    streaming,
    streamedText,
    error,
    cancel,
  };
}