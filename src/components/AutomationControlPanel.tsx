import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AutomationMessage {
  sender: string;
  text: string;
}

const AutomationControlPanel: React.FC = () => {
  const [coords, setCoords] = useState({ x: 0, y: 0 });
  const [drag, setDrag] = useState({ startX: 0, startY: 0, endX: 0, endY: 0 });
  const [scrollAmount, setScrollAmount] = useState(10);
  const [text, setText] = useState("");
  const [key, setKey] = useState("enter");
  const [status, setStatus] = useState<string | null>(null);
  const [messages, setMessages] = useState<AutomationMessage[]>([]);
  const [loadingAction, setLoadingAction] = useState<string | null>(null);

  const runCommand = async (name: string, payload: any) => {
    setLoadingAction(name);
    try {
      await invoke(name, payload);
      setStatus(`${name} succeeded`);
    } catch (err) {
      console.error(err);
      setStatus(`Error: ${String(err)}`);
    } finally {
      setLoadingAction(null);
    }
  };

  useEffect(() => {
    const unlistenAction = listen("ai-action-executed", (event) => {
      console.log("AI executed:", event.payload);
      setMessages(msgs => [...msgs, {
        sender: "system",
        text: `✓ ${event.payload}`
      }]);
    });

    const unlistenError = listen("ai-action-error", (event) => {
      console.log("AI action failed:", event.payload);
      setMessages(msgs => [...msgs, {
        sender: "system",
        text: `✗ Action failed: ${event.payload}`
      }]);
    });

    return () => {
      unlistenAction.then(f => f());
      unlistenError.then(f => f());
    };
  }, []);

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Mouse Controls */}
        <div className="space-y-2.5">
          <div className="text-xs text-blue-300 font-medium">Move mouse to:</div>
          <div className="flex gap-2">
            <input
              type="number"
              value={coords.x}
              onChange={(e) => setCoords((prev) => ({ ...prev, x: Number(e.target.value) }))}
              className="w-20 px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="x"
            />
            <input
              type="number"
              value={coords.y}
              onChange={(e) => setCoords((prev) => ({ ...prev, y: Number(e.target.value) }))}
              className="w-20 px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="y"
            />
            <button
              type="button"
              onClick={() => runCommand("mouse_move", { x: coords.x, y: coords.y })}
              disabled={loadingAction === "mouse_move"}
              className="px-3 py-1.5 rounded bg-blue-700 hover:bg-blue-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loadingAction === "mouse_move" ? "Moving..." : "Move"}
            </button>
          </div>

          <div className="text-xs text-blue-300 font-medium">Click:</div>
          <div className="flex gap-2 flex-wrap">
            {['left', 'right', 'middle'].map((button) => (
              <button
                key={button}
                type="button"
                onClick={() => runCommand("mouse_click", { button })}
                disabled={loadingAction === `click-${button}`}
                className="px-3 py-1.5 rounded bg-purple-700 hover:bg-purple-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors capitalize"
              >
                {loadingAction === `click-${button}` ? "Clicking..." : button}
              </button>
            ))}
          </div>
        </div>

        {/* Drag & Scroll */}
        <div className="space-y-2.5">
          <div className="text-xs text-blue-300 font-medium">Drag (from → to):</div>
          <div className="grid grid-cols-2 gap-2">
            <input
              type="number"
              value={drag.startX}
              onChange={(e) => setDrag((prev) => ({ ...prev, startX: Number(e.target.value) }))}
              className="w-full px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="from x"
            />
            <input
              type="number"
              value={drag.startY}
              onChange={(e) => setDrag((prev) => ({ ...prev, startY: Number(e.target.value) }))}
              className="w-full px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="from y"
            />
            <input
              type="number"
              value={drag.endX}
              onChange={(e) => setDrag((prev) => ({ ...prev, endX: Number(e.target.value) }))}
              className="w-full px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="to x"
            />
            <input
              type="number"
              value={drag.endY}
              onChange={(e) => setDrag((prev) => ({ ...prev, endY: Number(e.target.value) }))}
              className="w-full px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="to y"
            />
          </div>
          <button
            type="button"
            onClick={() => runCommand("mouse_drag", { startX: drag.startX, startY: drag.startY, endX: drag.endX, endY: drag.endY })}
            disabled={loadingAction === "mouse_drag"}
            className="w-full px-3 py-1.5 rounded bg-emerald-700 hover:bg-emerald-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {loadingAction === "mouse_drag" ? "Dragging..." : "Drag"}
          </button>
        </div>
      </div>

      <div className="border-t border-slate-700/50 pt-3" />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="space-y-2.5">
          <div className="text-xs text-blue-300 font-medium">Scroll:</div>
          <div className="flex gap-2 items-center">
            <input
              type="number"
              value={scrollAmount}
              onChange={(e) => setScrollAmount(Number(e.target.value))}
              className="w-20 px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
            />
            <button
              type="button"
              onClick={() => runCommand("mouse_scroll", { amount: scrollAmount })}
              disabled={loadingAction === "mouse_scroll"}
              className="px-3 py-1.5 rounded bg-indigo-700 hover:bg-indigo-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loadingAction === "mouse_scroll" ? "Scrolling..." : "Scroll"}
            </button>
          </div>
        </div>

        <div className="space-y-2.5">
          <div className="text-xs text-blue-300 font-medium">Type text:</div>
          <div className="flex gap-2">
            <input
              type="text"
              value={text}
              onChange={(e) => setText(e.target.value)}
              className="flex-1 px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="text to type"
            />
            <button
              type="button"
              onClick={() => runCommand("type_text", { text })}
              disabled={loadingAction === "type_text"}
              className="px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loadingAction === "type_text" ? "Typing..." : "Type"}
            </button>
          </div>
          <div className="text-xs text-blue-300 font-medium">Press key:</div>
          <div className="flex gap-2">
            <input
              type="text"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              className="flex-1 px-2 py-1.5 rounded bg-slate-800 border border-slate-700 text-white text-xs focus:border-blue-500 focus:outline-none transition-colors"
              placeholder="enter, tab..."
            />
            <button
              type="button"
              onClick={() => runCommand("press_key", { key })}
              disabled={loadingAction === "press_key"}
              className="px-3 py-1.5 rounded bg-fuchsia-700 hover:bg-fuchsia-600 text-white text-xs disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loadingAction === "press_key" ? "Pressing..." : "Press"}
            </button>
          </div>
        </div>
      </div>

      {/* Status messages */}
      {(status || messages.length > 0) && (
        <div className="border-t border-slate-700/50 pt-3 space-y-1">
          {status && <div className="text-xs text-blue-200">{status}</div>}
          {messages.map((msg, idx) => (
            <div key={idx} className="text-xs text-blue-200">{msg.text}</div>
          ))}
        </div>
      )}
    </div>
  );
};

export default AutomationControlPanel;