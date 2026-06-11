import { useCallback, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MemorySummary } from "../types";

interface UseStashStatus {
  stashAvailable: boolean | null;
  summary: MemorySummary | null;
  loading: boolean;
  checkStatus: () => Promise<void>;
}

export function useStashStatus(): UseStashStatus {
  const [stashAvailable, setStashAvailable] = useState<boolean | null>(null);
  const [summary, setSummary] = useState<MemorySummary | null>(null);
  const [loading, setLoading] = useState(false);

  const checkStatus = useCallback(async (): Promise<void> => {
    setLoading(true);
    try {
      const result = await invoke<MemorySummary>("memory_summary");
      setSummary(result);
      setStashAvailable(true);
    } catch (err) {
      console.warn("[Stash] memory_summary failed — marking Stash as unavailable:", err);
      setStashAvailable(false);
      setSummary(null);
    } finally {
      setLoading(false);
    }
  }, []);

  // Auto-check on mount
  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  return { stashAvailable, summary, loading, checkStatus };
}