import React, { useState } from "react";
import FileLinker from "./FileLinker";
import VoiceControls from "./VoiceControls";

interface Props {
  onSend: (text: string) => void;
  disabled?: boolean;
  onFilesSelected?: (files: { name: string; content: string }[] | null) => void;
  isListening?: boolean;
  isSpeaking?: boolean;
  onVoiceInput?: (text: string) => void;
  onVoiceOutput?: (text: string) => void;
}

const MessageInput: React.FC<Props> = ({ onSend, disabled, onFilesSelected, isListening = false, isSpeaking = false, onVoiceInput, onVoiceOutput }) => {
  const [value, setValue] = useState("");

  const handleSend = (e: React.FormEvent) => {
    e.preventDefault();
    if (!value.trim()) return;
    onSend(value.trim());
    setValue("");
  };

  const handleVoiceInput = (text: string) => {
    if (text.trim()) {
      onSend(text.trim());
    }
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
    <div className="w-full bg-slate-900/40">
      {/* Markdown Toolbar Hint */}
      <div className="px-4 py-1.5 bg-slate-800/30 text-[10px] text-slate-500 border-b border-slate-800/30 flex items-center justify-between">
        <span>Supports Markdown, LaTeX ($...$), and code blocks</span>
        <span className="opacity-60">Enter to send, Shift+Enter for newline</span>
      </div>

      {/* Formatting toolbar */}
      <div className="flex flex-wrap gap-1 px-4 py-2 bg-[#232a4a] border-b border-slate-800/30">
        {toolbarButtons.map((btn, idx) => (
          <button
            key={idx}
            type="button"
            onClick={btn.action}
            className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-slate-200 rounded transition-colors font-medium active:scale-95"
            title={btn.title}
          >
            {btn.label}
          </button>
        ))}
      </div>

      {/* Main input area */}
      <form
        onSubmit={handleSend}
        className="w-full flex items-end gap-3 px-4 py-3 bg-slate-800/20 relative"
      >
        <div className="flex items-center gap-2 shrink-0">
          <FileLinker onFilesSelected={onFilesSelected} />
          {onVoiceInput && (
            <VoiceControls
              onVoiceInput={handleVoiceInput}
              onVoiceOutput={onVoiceOutput}
              isListening={isListening}
              isSpeaking={isSpeaking}
            />
          )}
        </div>

        <textarea
          className="flex-1 bg-[#232a4a] text-white px-4 py-2.5 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 resize-none font-mono text-sm leading-relaxed placeholder:text-slate-500 transition-all shadow-inner"
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
          className="shrink-0 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 text-white px-5 py-2.5 rounded-xl font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-all shadow-lg hover:shadow-blue-500/25"
          disabled={disabled || !value.trim()}
        >
          {disabled ? (
            <>
              <span className="animate-pulse">●</span> Sending...
            </>
          ) : (
            <>
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
              </svg>
              Send
            </>
          )}
        </button>
      </form>
    </div>
  );
};

export default MessageInput;