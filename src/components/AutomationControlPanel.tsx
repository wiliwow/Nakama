import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const AutomationControlPanel: React.FC = () => {
  const [coords, setCoords] = useState({ x: 0, y: 0 });
  const [drag, setDrag] = useState({ startX: 0, startY: 0, endX: 0, endY: 0 });
  const [scrollAmount, setScrollAmount] = useState(10);
  const [text, setText] = useState("");
  const [key, setKey] = useState("enter");
  const [status, setStatus] = useState<string | null>(null);

  const runCommand = async (name: string, payload: any) => {
    try {
      await invoke(name, payload);
      setStatus(`${name} succeeded`);
    } catch (err) {
      console.error(err);
      setStatus(`Error: ${String(err)}`);
    }
  };

  return (
    <div className="bg-[#1b2140] border border-blue-900 rounded-2xl p-4 space-y-3">
      <div className="text-sm font-semibold text-yellow-200">Automation Controls</div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="space-y-2">
          <div className="text-xs text-blue-300">Move mouse to:</div>
          <div className="flex gap-2">
            <input
              type="number"
              value={coords.x}
              onChange={(e) => setCoords((prev) => ({ ...prev, x: Number(e.target.value) }))}
              className="w-20 px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="x"
            />
            <input
              type="number"
              value={coords.y}
              onChange={(e) => setCoords((prev) => ({ ...prev, y: Number(e.target.value) }))}
              className="w-20 px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="y"
            />
            <button
              type="button"
              onClick={() => runCommand("mouse_move", { x: coords.x, y: coords.y })}
              className="px-3 py-2 rounded bg-blue-700 hover:bg-blue-600 text-white text-xs"
            >
              Move
            </button>
          </div>
          <div className="text-xs text-blue-300">Click:</div>
          <div className="flex gap-2 flex-wrap">
            {['left', 'right', 'middle'].map((button) => (
              <button
                key={button}
                type="button"
                onClick={() => runCommand("mouse_click", { button })}
                className="px-3 py-2 rounded bg-purple-700 hover:bg-purple-600 text-white text-xs"
              >
                {button}
              </button>
            ))}
          </div>
          <div className="text-xs text-blue-300">Drag:</div>
          <div className="grid grid-cols-2 gap-2">
            <input
              type="number"
              value={drag.startX}
              onChange={(e) => setDrag((prev) => ({ ...prev, startX: Number(e.target.value) }))}
              className="w-full px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="from x"
            />
            <input
              type="number"
              value={drag.startY}
              onChange={(e) => setDrag((prev) => ({ ...prev, startY: Number(e.target.value) }))}
              className="w-full px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="from y"
            />
            <input
              type="number"
              value={drag.endX}
              onChange={(e) => setDrag((prev) => ({ ...prev, endX: Number(e.target.value) }))}
              className="w-full px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="to x"
            />
            <input
              type="number"
              value={drag.endY}
              onChange={(e) => setDrag((prev) => ({ ...prev, endY: Number(e.target.value) }))}
              className="w-full px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="to y"
            />
          </div>
          <button
            type="button"
            onClick={() => runCommand("mouse_drag", { startX: drag.startX, startY: drag.startY, endX: drag.endX, endY: drag.endY })}
            className="px-3 py-2 rounded bg-emerald-700 hover:bg-emerald-600 text-white text-xs"
          >
            Drag
          </button>
        </div>

        <div className="space-y-2">
          <div className="text-xs text-blue-300">Scroll:</div>
          <div className="flex gap-2 items-center">
            <input
              type="number"
              value={scrollAmount}
              onChange={(e) => setScrollAmount(Number(e.target.value))}
              className="w-20 px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
            />
            <button
              type="button"
              onClick={() => runCommand("mouse_scroll", { amount: scrollAmount })}
              className="px-3 py-2 rounded bg-indigo-700 hover:bg-indigo-600 text-white text-xs"
            >
              Scroll
            </button>
          </div>
          <div className="text-xs text-blue-300">Type text:</div>
          <div className="flex gap-2">
            <input
              type="text"
              value={text}
              onChange={(e) => setText(e.target.value)}
              className="flex-1 px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="text to type"
            />
            <button
              type="button"
              onClick={() => runCommand("type_text", { text })}
              className="px-3 py-2 rounded bg-sky-700 hover:bg-sky-600 text-white text-xs"
            >
              Type
            </button>
          </div>
          <div className="text-xs text-blue-300">Press key:</div>
          <div className="flex gap-2 items-center">
            <input
              type="text"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              className="flex-1 px-2 py-1 rounded bg-[#15203a] border border-blue-800 text-white text-xs"
              placeholder="enter, tab, space..."
            />
            <button
              type="button"
              onClick={() => runCommand("press_key", { key })}
              className="px-3 py-2 rounded bg-fuchsia-700 hover:bg-fuchsia-600 text-white text-xs"
            >
              Press
            </button>
          </div>
        </div>
      </div>

      {status && <div className="text-xs text-blue-200">{status}</div>}
    </div>
  );
};

export default AutomationControlPanel;
