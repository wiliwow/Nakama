import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface RecordingControlsProps {
  onStatusChange?: (recording: boolean) => void;
}

interface RecordingSummary {
  video_path: string;
  keylog_path: string;
  duration_seconds: number;
}

const RecordingControls: React.FC<RecordingControlsProps> = ({ onStatusChange }) => {
  const [recording, setRecording] = useState(false);
  const [time, setTime] = useState(0);
  const [lastRecording, setLastRecording] = useState<RecordingSummary | null>(null);
  const [showPopup, setShowPopup] = useState(false);

  // Poll recording status
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const status = await invoke<boolean>("get_screen_recording_status");
        setRecording(status);
        onStatusChange?.(status);
      } catch (err) {
        console.error("Error polling recording status:", err);
      }
    }, 500);
    return () => clearInterval(interval);
  }, [onStatusChange]);

  // Update timer
  useEffect(() => {
    const interval = setInterval(() => {
      if (recording) setTime(t => t + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [recording]);

  const handleToggle = async () => {
    try {
      if (recording) {
        const res = await invoke<string>("stop_screen_recording");
        try {
          const parsed = typeof res === "string" ? JSON.parse(res) : res;
          if (parsed && parsed.video_path) {
            setLastRecording(parsed as RecordingSummary);
            setShowPopup(true);
            setTimeout(() => setShowPopup(false), 8000);
          }
        } catch (e) {
          // ignore non-json
        }
        setTime(0);
      } else {
        await invoke("start_screen_recording");
      }
    } catch (err) {
      console.error("Recording error:", err);
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:
${secs.toString().padStart(2, "0")}`.replace(/\n/, "");
  };

  return (
    <div className="flex gap-4 p-4 bg-[#181e36]/60 rounded-lg border border-blue-900/50">
      <div className="flex flex-col items-center">
        <button
          onClick={handleToggle}
          className={`px-4 py-2 rounded-lg font-semibold transition-all ${
            recording
              ? "bg-red-600 hover:bg-red-700 text-white shadow-lg shadow-red-500/50"
              : "bg-gray-700 hover:bg-gray-600 text-white"
          }`}
        >
          {recording ? "🔴 Stop Recording" : "🎬 Record (Video+Audio+Input)"}
        </button>
        {recording && <span className="text-sm text-red-400 mt-1 font-mono">{formatTime(time)}</span>}
      </div>

      <div className="flex items-center gap-2 ml-auto px-4">
        {recording && (
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-red-500 rounded-full animate-pulse"></div>
            <span className="text-sm text-red-400">Recording</span>
          </div>
        )}
      </div>

      {/* Sliding Popup */}
      {lastRecording && (
        <div
          className={`fixed right-6 top-6 z-50 transform transition-all duration-300 ${
            showPopup ? "opacity-100 translate-y-0" : "opacity-0 -translate-y-4 pointer-events-none"
          }`}
        >
          <div className="max-w-sm w-80 bg-gray-900/95 border border-blue-800/60 text-white rounded-lg p-4 shadow-lg">
            <div className="flex items-start gap-3">
              <div className="flex-1">
                <div className="font-semibold">Recording Saved</div>
                <div className="text-xs text-gray-300 mt-1">Duration: {formatTime(lastRecording.duration_seconds)}</div>
                <div className="text-xs text-gray-300 mt-2 break-all">Video: {lastRecording.video_path}</div>
                <div className="text-xs text-gray-300 mt-1 break-all">Keylog: {lastRecording.keylog_path}</div>
              </div>
              <button onClick={() => setShowPopup(false)} className="text-gray-400 hover:text-gray-200" aria-label="Close">
                ✕
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default RecordingControls;
