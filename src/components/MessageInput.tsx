import React, { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import FileLinker from "./FileLinker";

interface Props {
  onSend: (text: string) => void;
  disabled?: boolean;
  onFilesSelected?: (files: string[] | null) => void;
}

const MessageInput: React.FC<Props> = ({ onSend, disabled, onFilesSelected }) => {
  const [value, setValue] = useState("");

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault();
    if (value.trim()) {
      onSend(value);
      invoke("get_message", { message: value.trim() });
      setValue("");
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();   // Prevent a newline from being added
      handleSend(e);        // Submit the form
    }
    // If Shift+Enter is pressed, the default behavior (newline) is preserved
  };

  return (
    <form
      onSubmit={handleSend}
      className="w-full flex items-center gap-2 px-4 py-3 bg-[#181e36] rounded-b-2xl shadow-inner relative"
    >
      <div className="mr-2">
        <FileLinker onFilesSelected={onFilesSelected} />
      </div>
      <textarea
        className="flex-1 bg-[#232a4a] text-white px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none"
        placeholder="Type your message..."
        value={value}
        onChange={e => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        autoFocus
        rows={1}            // optional: start with a single row
      />
      <button
        type="submit"
        className="bg-blue-700 hover:bg-blue-600 text-white px-4 py-2 rounded-lg font-semibold disabled:opacity-50"
        disabled={disabled || !value.trim()}
      >
        Send
      </button>
    </form>
  );
};

export default MessageInput;