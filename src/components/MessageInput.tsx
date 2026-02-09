import React, { useState } from "react";
import { invoke } from '@tauri-apps/api/core';

interface Props {
  onSend: (text: string) => void;
  disabled?: boolean;
}

const MessageInput: React.FC<Props> = ({ onSend, disabled }) => {
  const [value, setValue] = useState("");

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault();
    if (value.trim()) {
      onSend(value);
      invoke("get_message", { message: value.trim() });
      setValue("");
    }
  };

  return (
    <>
      <form
        onSubmit={handleSend}
        className="w-full flex items-center gap-2 px-4 py-3 bg-[#181e36] rounded-b-2xl shadow-inner relative"
      >
        <input
          className="flex-1 bg-[#232a4a] text-white px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400"
          type="text"
          placeholder="Type your message..."
          value={value}
          onChange={e => setValue(e.target.value)}
          disabled={disabled}
          autoFocus
          onKeyDown={e => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              handleSend(e);
            }
          }}
        />
        <button
          type="submit"
          className="bg-blue-700 hover:bg-blue-600 text-white px-4 py-2 rounded-lg font-semibold disabled:opacity-50"
          disabled={disabled || !value.trim()}
        >
          Send
        </button>
      </form>
    </>
  );
};

export default MessageInput;
