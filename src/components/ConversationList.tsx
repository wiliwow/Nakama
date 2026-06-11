import React from "react";
import { useFocusTrap } from "../hooks/useFocusTrap";

interface Conversation {
  id?: number;
  title: string;
  created_at: string;
  updated_at: string;
}

interface ConversationListProps {
  conversations: Conversation[];
  onSelect: (id: number) => void;
  onClose: () => void;
}

const ConversationList: React.FC<ConversationListProps> = ({ conversations, onSelect, onClose }) => {
  useFocusTrap(true);
  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm animate-fade-in"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="absolute left-0 right-0 top-[5.5rem] mx-auto z-50 w-full max-w-[1500px] px-4 animate-slide-in">
        <div className="rounded-[32px] border border-slate-800/60 bg-slate-950/95 shadow-2xl overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-slate-800/50">
            <h2 className="text-sm font-semibold text-slate-200">Recent Conversations</h2>
            <button
              onClick={onClose}
              className="p-1 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
              aria-label="Close"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div className="max-h-64 overflow-y-auto p-4">
            {conversations.length === 0 ? (
              <div className="text-xs italic text-slate-500 text-center py-8">
                No conversations yet
              </div>
            ) : (
              <div className="space-y-2">
                {conversations.map((conv) => (
                  <button
                    key={conv.id}
                    onClick={() => onSelect(conv.id!)}
                    className="w-full rounded-2xl border border-slate-800/60 bg-slate-900/60 px-4 py-3 text-left transition-all hover:border-blue-500/50 hover:bg-slate-800/80 group"
                  >
                    <div className="text-sm text-slate-100 truncate group-hover:text-blue-300 transition-colors">
                      {conv.title}
                    </div>
                    <div className="text-xs text-slate-500 mt-1">
                      {new Date(conv.updated_at).toLocaleDateString()}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
};

export default ConversationList;
