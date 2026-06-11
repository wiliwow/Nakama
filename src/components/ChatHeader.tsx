import React from "react";
import Button from "./ui/Button";
import Badge from "./ui/Badge";
import Toggle from "./ui/Toggle";

interface ChatHeaderProps {
  indexedFilesCount: number;
  voiceConversationEnabled: boolean;
  onNewConversation: () => void;
  onToggleConversationList: () => void;
  onToggleVoiceChat: (checked: boolean) => void;
  onToggleMemory: () => void;
  memoryCount: { episodes: number; facts: number; goals: number };
  localMemoryAvailable: boolean;
  currentConversationTitle?: string;
}

const ChatHeader: React.FC<ChatHeaderProps> = ({
  indexedFilesCount,
  voiceConversationEnabled,
  onNewConversation,
  onToggleConversationList,
  onToggleVoiceChat,
  onToggleMemory,
  memoryCount,
  localMemoryAvailable,
  currentConversationTitle,
}) => {
  return (
    <div className="flex flex-col gap-3 border-b border-slate-800/30 bg-gradient-to-b from-slate-900/50 to-slate-950/30 backdrop-blur-sm">
      <div className="px-4 pt-4 pb-2 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-10 h-10 rounded-full bg-gradient-to-br from-blue-600 to-purple-700 shadow-lg">
            <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <div>
            <h1 className="text-xl font-bold text-white tracking-wide">Nakama</h1>
            <p className="text-xs text-slate-400">Your friendly AI companion</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Badge count={indexedFilesCount} variant="success" />
          <div className="flex items-center gap-2 pl-2 border-l border-slate-700/50">
            <span className="text-xs text-slate-400 flex items-center gap-1">
              <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
              </svg>
              Voice
            </span>
            <Toggle
              checked={voiceConversationEnabled}
              onChange={onToggleVoiceChat}
            />
          </div>

          <button
            onClick={onToggleMemory}
            className={`px-3 py-1.5 rounded-lg text-sm font-semibold transition-all flex items-center gap-1.5 ${
              localMemoryAvailable
                ? "bg-blue-600/20 text-blue-300 hover:bg-blue-600/30 border border-blue-700/50"
                : "bg-slate-700 text-slate-400 cursor-not-allowed"
            }`}
            title="Memory"
          >
            <span className="text-base">🧠</span>
            Memory
            {localMemoryAvailable && memoryCount.episodes > 0 && (
              <span className="bg-blue-500 text-white text-[10px] px-1 rounded-full">
                {memoryCount.episodes}
              </span>
            )}
          </button>
        </div>
      </div>

      <div className="px-4 pb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button variant="primary" size="sm" onClick={onNewConversation}>
            <span className="mr-1">+</span> New Chat
          </Button>
          <Button variant="secondary" size="sm" onClick={onToggleConversationList}>
            <span className="mr-1">📂</span> Load Chat
          </Button>
        </div>

        {currentConversationTitle && (
          <div className="text-sm text-slate-400 truncate max-w-xs border-l border-slate-700/50 pl-3">
            {currentConversationTitle}
          </div>
        )}
      </div>
    </div>
  );
};

export default ChatHeader;
