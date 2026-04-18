import React, { useState } from "react";
import FileLinker from "./FileLinker";

interface Props {
  onSend: (text: string) => void;
  disabled?: boolean;
  onFilesSelected?: (files: { name: string; content: string }[] | null) => void;
}

const MessageInput: React.FC<Props> = ({ onSend, disabled, onFilesSelected }) => {
  const [value, setValue] = useState("");

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault();
    if (!value.trim()) return;
    onSend(value.trim());
    setValue("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();   // Prevent a newline from being added
      handleSend(e);        // Submit the form
    }
    // If Shift+Enter is pressed, the default behavior (newline) is preserved
  };

  const insertMarkdown = (before: string, after: string = "", placeholder: string = "text") => {
    const textarea = document.querySelector('textarea') as HTMLTextAreaElement;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = value.substring(start, end);
    const replacement = selectedText || placeholder;
    const newText = value.substring(0, start) + before + replacement + after + value.substring(end);
    setValue(newText);

    // Focus back to textarea and set cursor position
    setTimeout(() => {
      textarea.focus();
      const newCursorPos = start + before.length + replacement.length;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  const toolbarButtons = [
    { label: 'B', title: 'Bold', action: () => insertMarkdown('**', '**', 'bold text') },
    { label: 'I', title: 'Italic', action: () => insertMarkdown('*', '*', 'italic text') },
    { label: '</>', title: 'Code', action: () => insertMarkdown('`', '`', 'code') },
    { label: '∑', title: 'Math', action: () => insertMarkdown('$$', '$$', 'math expression') },
    { label: 'H1', title: 'Heading 1', action: () => insertMarkdown('# ', '', 'Heading') },
    { label: 'H2', title: 'Heading 2', action: () => insertMarkdown('## ', '', 'Heading') },
    { label: '•', title: 'Bullet List', action: () => insertMarkdown('- ', '', 'List item') },
    { label: '1.', title: 'Numbered List', action: () => insertMarkdown('1. ', '', 'List item') },
    { label: '🔗', title: 'Link', action: () => insertMarkdown('[', '](url)', 'link text') },
    { label: '```', title: 'Code Block', action: () => insertMarkdown('```\n', '\n```', 'code block') },
    { label: '>', title: 'Quote', action: () => insertMarkdown('> ', '', 'quote') },
  ];

  return (
    <div className="w-full">
      {/* Markdown Toolbar */}
      <div className="px-4 py-1 bg-[#232a4a] border-b border-gray-600 text-xs text-gray-400 text-center">
        Supports plain text, Markdown, LaTeX ($...$ or $$...$$), and code blocks.
      </div>
      <div className="flex flex-wrap gap-1 px-4 py-2 bg-[#232a4a] rounded-t-2xl border-b border-gray-600">
        {toolbarButtons.map((btn, idx) => (
          <button
            key={idx}
            type="button"
            onClick={btn.action}
            className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 text-gray-200 rounded transition-colors font-medium"
            title={btn.title}
          >
            {btn.label}
          </button>
        ))}
      </div>

      <form
        onSubmit={handleSend}
        className="w-full flex items-end gap-2 px-4 py-3 bg-[#181e36] rounded-b-2xl shadow-inner relative"
      >
        <div className="mr-2 flex items-center gap-2">
          <FileLinker onFilesSelected={onFilesSelected} />
        </div>
        <textarea
          className="flex-1 bg-[#232a4a] text-white px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none font-mono"
          placeholder="Type your message... markdown, LaTeX, or plain text all supported"
          value={value}
          onChange={e => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          autoFocus
          rows={3}
          style={{ minHeight: '80px' }}
        />
        <button
          type="submit"
          className="bg-blue-700 hover:bg-blue-600 text-white px-4 py-2 rounded-lg font-semibold disabled:opacity-50"
          disabled={disabled || !value.trim()}
        >
          Send
        </button>
      </form>
    </div>
  );
};

export default MessageInput;