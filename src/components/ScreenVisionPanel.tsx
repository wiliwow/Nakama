import React from "react";

export interface ScreenCapture {
  id: number;
  width: number;
  height: number;
  dataUrl: string;
}

interface Props {
  enabled: boolean;
  captures: ScreenCapture[];
  onToggleEnabled: () => void;
  onCapturePrimary: () => void;
  onCaptureAll: () => void;
  loading?: "primary" | "all" | null;
}

const ScreenVisionPanel: React.FC<Props> = ({ enabled, captures, onToggleEnabled, onCapturePrimary, onCaptureAll, loading }) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-xs text-blue-300">Capture your display and let the assistant inspect the current screen.</div>
        </div>
        <button
          type="button"
          onClick={onToggleEnabled}
          className={`px-3 py-1 rounded text-xs font-semibold transition ${enabled ? 'bg-green-600 text-white' : 'bg-slate-700 text-slate-300 hover:bg-slate-600'}`}
        >
          {enabled ? "Enabled" : "Disabled"}
        </button>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={onCapturePrimary}
          disabled={loading === "primary"}
          className="px-3 py-2 rounded bg-blue-700 hover:bg-blue-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loading === "primary" ? "Capturing..." : "Capture Primary Screen"}
        </button>
        <button
          type="button"
          onClick={onCaptureAll}
          disabled={loading === "all"}
          className="px-3 py-2 rounded bg-purple-700 hover:bg-purple-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loading === "all" ? "Capturing..." : "Capture All Screens"}
        </button>
      </div>

      {captures.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {captures.map((capture) => (
            <div key={capture.id} className="rounded-lg border border-blue-800/50 overflow-hidden bg-black">
              <div className="px-3 py-2 bg-[#111827] text-xs text-blue-200">
                Screen {capture.id + 1} • {capture.width}×{capture.height}
              </div>
              <img src={capture.dataUrl} alt={`Screenshot ${capture.id}`} className="w-full h-auto object-contain" />
            </div>
          ))}
        </div>
      ) : (
        <div className="text-xs text-blue-400 text-center py-4 bg-blue-900/10 rounded-lg border border-blue-900/30">
          No screen capture yet. Use the buttons above to grab the current display.
        </div>
      )}
    </div>
  );
};

export default ScreenVisionPanel;
