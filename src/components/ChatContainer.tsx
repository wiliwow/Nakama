import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";
import AutomationControlPanel from "./AutomationControlPanel";
import ScreenVisionPanel, { ScreenCapture } from "./ScreenVisionPanel";

interface RetrievedPassage {
  id: string;
  score: number;
  content: string;
  source?: string;
}

interface ConversationMessage {
  id?: number;
  role: string;
  content: string;
  timestamp: string;
  metadata?: any;
}

interface Conversation {
  id?: number;
  title: string;
  created_at: string;
  updated_at: string;
  messages: ConversationMessage[];
}

const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" },
  ]);
  const [loading, setLoading] = useState(false);
  const [files, setFiles] = useState<{ name: string; content: string }[]>([]);
  const [indexingFiles, setIndexingFiles] = useState(false);
  const [indexedFilesCount, setIndexedFilesCount] = useState(0);
  const [screenVisionEnabled, setScreenVisionEnabled] = useState(false);
  const [screenCaptures, setScreenCaptures] = useState<ScreenCapture[]>([]);
  const [screenCaptureStatus, setScreenCaptureStatus] = useState<string | null>(null);

  const [currentConversation, setCurrentConversation] = useState<Conversation | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [showConversationList, setShowConversationList] = useState(false);

  const handleFilesSelected = (selectedFiles: { name: string; content: string }[] | null) => {
    if (!selectedFiles) return;
    setFiles(prev => [...prev, ...selectedFiles]);
  };

  const capturePrimaryScreen = async () => {
    try {
      const capture = await invoke<ScreenCapture>("capture_primary_screen");
      setScreenCaptures([capture]);
      setScreenCaptureStatus("Primary screen captured.");
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: "I can now see your primary screen.", screenshots: [capture] },
      ]);
    } catch (err) {
      console.error("Failed to capture primary screen:", err);
      setScreenCaptureStatus(`Capture failed: ${String(err)}`);
    }
  };

  const captureAllScreens = async () => {
    try {
      const captures = await invoke<ScreenCapture[]>("capture_all_screens");
      setScreenCaptures(captures);
      setScreenCaptureStatus("All screens captured.");
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: "I can now see all your screens.", screenshots: captures },
      ]);
    } catch (err) {
      console.error("Failed to capture all screens:", err);
      setScreenCaptureStatus(`Capture failed: ${String(err)}`);
    }
  };

  const loadConversations = async () => {
    try {
      const convs = await invoke<Conversation[]>("conversation_list");
      setConversations(convs);
    } catch (err) {
      console.error("Failed to load conversations:", err);
    }
  };

  const createNewConversation = async () => {
    try {
      const title = `Conversation ${new Date().toLocaleString()}`;
      const conv = await invoke<Conversation>("conversation_create", { title });
      setCurrentConversation(conv);
      setMessages([{ sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" }]);
      await loadConversations();
    } catch (err) {
      console.error("Failed to create conversation:", err);
    }
  };

  const loadConversation = async (id: number) => {
    try {
      const conv = await invoke<Conversation | null>("conversation_load", { id });
      if (conv) {
        setCurrentConversation(conv);
        setMessages(
          conv.messages.map(msg => ({
            sender: msg.role === "user" ? "user" : "ai",
            text: msg.content,
          }))
        );
        setShowConversationList(false);
      }
    } catch (err) {
      console.error("Failed to load conversation:", err);
    }
  };

  const saveCurrentConversation = async () => {
    if (!currentConversation) return;

    try {
      const convMessages: ConversationMessage[] = messages
        .filter(msg => msg.sender !== "ai" || msg.text !== "Hi! I'm Nakama, your night-sky companion. How can I help you today?")
        .map(msg => ({ role: msg.sender, content: msg.text, timestamp: new Date().toISOString() }));

      const updatedConv = {
        ...currentConversation,
        messages: convMessages,
        updated_at: new Date().toISOString(),
      };

      await invoke("conversation_save", { conversation: updatedConv });
      setCurrentConversation(updatedConv);
    } catch (err) {
      console.error("Failed to save conversation:", err);
    }
  };

  useEffect(() => {
    loadConversations();
  }, []);

  const indexStagedFiles = async () => {
    if (files.length === 0) return;
    try {
      setIndexingFiles(true);
      let successCount = 0;
      for (const file of files) {
        try {
          const result = await invoke<string>("rag_add_file", { filename: file.name, content: file.content });
          console.log(`[RAG] Successfully indexed: ${result}`);
          successCount++;
        } catch (err) {
          console.error(`[RAG] Failed to index ${file.name}:`, err);
        }
      }
      setIndexedFilesCount(prev => prev + successCount);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `✓ Indexed ${successCount}/${files.length} file(s) to memory. You can now ask questions about them!` },
      ]);
      setFiles([]);
    } catch (err) {
      console.error("[RAG] Error during indexing:", err);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `⚠️ Error indexing files: ${String(err)}. Make sure the embedding engine is running.` },
      ]);
    } finally {
      setIndexingFiles(false);
    }
  };

  const handleSend = async (text: string) => {
    if (!text.trim()) return;
    setMessages(msgs => [...msgs, { sender: "user", text, files: files.length ? files : undefined }]);
    setLoading(true);
    setMessages(msgs => [...msgs, { sender: "ai", text: "" }]);

    if (!currentConversation) {
      await createNewConversation();
    }

    try {
      const history: ConversationMessage[] = messages
        .filter(msg => msg.sender !== "ai" || msg.text !== "Hi! I'm Nakama, your night-sky companion. How can I help you today?")
        .map(msg => ({ role: msg.sender, content: msg.text, timestamp: new Date().toISOString() }));

      if (screenVisionEnabled && screenCaptures.length > 0) {
        history.push({ role: "system", content: "Screen vision is enabled and the latest display capture is available in the UI.", timestamp: new Date().toISOString() });
      }

      if (indexedFilesCount > 0) {
        try {
          const retrieved = await invoke<RetrievedPassage[]>("rag_retrieve", { query_text: text, top_k: 10 });
          if (retrieved && retrieved.length > 0) {
            history.push({
              role: "system",
              content: `Retrieved context from indexed files:\n\n${retrieved
                .map((r, idx) => {
                  const src = r.source ? `Source: ${r.source}` : `Source ${idx + 1}`;
                  return `[${src} - Relevance: ${(r.score * 100).toFixed(1)}%]\n${r.content}`;
                })
                .join("\n\n---\n\n")}`,
              timestamp: new Date().toISOString(),
            });
          }
        } catch (ragErr) {
          console.warn("[AI] RAG retrieval failed:", ragErr);
        }
      }

      if (files.length > 0) {
        await indexStagedFiles();
      }

      const unlistenChunk = await listen<string>("ai-stream-chunk", (event) => {
        setMessages(msgs => {
          const copy = [...msgs];
          const lastIdx = copy.length - 1;
          if (lastIdx >= 0 && copy[lastIdx].sender === "ai") {
            copy[lastIdx] = { ...copy[lastIdx], text: copy[lastIdx].text + (event.payload ?? "") };
          }
          return copy;
        });
      });

      const unlistenError = await listen<string>("ai-stream-error", (event) => {
        setMessages(msgs => {
          const copy = [...msgs];
          const lastIdx = copy.length - 1;
          if (lastIdx >= 0 && copy[lastIdx].sender === "ai") {
            copy[lastIdx] = { ...copy[lastIdx], text: `Error: ${event.payload}` };
          }
          return copy;
        });
      });

      const unlistenDone = await listen<void>("ai-stream-done", async () => {
        setLoading(false);
        await saveCurrentConversation();
        await unlistenChunk();
        await unlistenError();
        await unlistenDone();
      });

      await invoke("ask_ai_stream_with_conversation", { prompt: text, conversationHistory: history });
    } catch (err) {
      console.error("[AI] Exception caught in handleSend:", err);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `Sorry, I encountered an error: ${String(err)}. Make sure Ollama is running locally.` },
      ]);
      setLoading(false);
    }
  };

  return (
    <div className="h-full min-h-0 flex flex-col px-4 py-4">
      <div className="mx-auto flex h-full w-full max-w-[1500px] flex-col min-h-0 overflow-hidden rounded-[32px] border border-slate-800 bg-slate-950/95 shadow-2xl">
        <div className="flex flex-col gap-3 border-b border-slate-800 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex flex-wrap items-center gap-2">
            <button
              onClick={createNewConversation}
              className="rounded-2xl bg-blue-700 px-4 py-2 text-xs font-semibold text-white transition hover:bg-blue-600"
            >
              ➕ New
            </button>
            <button
              onClick={() => setShowConversationList(prev => !prev)}
              className="rounded-2xl bg-violet-700 px-4 py-2 text-xs font-semibold text-white transition hover:bg-violet-600"
            >
              📂 Load
            </button>
            <span className="text-xs text-slate-400">{indexedFilesCount > 0 ? `${indexedFilesCount} indexed file(s)` : ""}</span>
          </div>
          {currentConversation && (
            <div className="text-xs text-slate-300 truncate max-w-xs">{currentConversation.title}</div>
          )}
        </div>

        <div className="flex-1 min-h-0 overflow-hidden md:flex md:flex-row">
          <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
            <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4">
              <MessageList messages={messages} />
            </div>

            {files.length > 0 && (
              <div className="border-t border-slate-800 bg-slate-950/90 px-4 py-3">
                <div className="text-xs text-slate-300 mb-2">Staged Files ({files.length}):</div>
                <div className="flex flex-wrap gap-2 mb-3">
                  {files.map((file, index) => (
                    <span key={index} className="rounded-full bg-slate-800 px-3 py-1 text-xs text-slate-200">
                      📄 {file.name}
                    </span>
                  ))}
                </div>
                <button
                  onClick={indexStagedFiles}
                  disabled={indexingFiles || loading}
                  className="rounded-2xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white transition hover:bg-emerald-500 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {indexingFiles ? "Indexing..." : "Index Files"}
                </button>
              </div>
            )}

            <div className="border-t border-slate-800 bg-slate-950/90 px-4 py-4">
              <MessageInput onSend={handleSend} disabled={loading || indexingFiles} onFilesSelected={handleFilesSelected} />
            </div>
          </div>

          <aside className="hidden min-h-0 w-full flex-col border-l border-slate-800 bg-slate-900/90 p-4 md:flex md:w-[360px]">
            <div className="min-h-0 flex-1 overflow-y-auto">
              <ScreenVisionPanel
                enabled={screenVisionEnabled}
                captures={screenCaptures}
                onToggleEnabled={() => setScreenVisionEnabled(prev => !prev)}
                onCapturePrimary={capturePrimaryScreen}
                onCaptureAll={captureAllScreens}
              />
            </div>
            {screenCaptureStatus && <div className="pt-3 text-xs text-slate-400">{screenCaptureStatus}</div>}
            <div className="min-h-0 flex-1 overflow-y-auto pt-4">
              <AutomationControlPanel />
            </div>
          </aside>
        </div>
      </div>

      {showConversationList && (
        <div className="absolute inset-x-0 top-[5.5rem] mx-auto w-full max-w-[1500px] px-4">
          <div className="rounded-[32px] border border-slate-800 bg-slate-950/95 shadow-2xl">
            <div className="border-b border-slate-800 px-4 py-3 text-sm font-semibold text-slate-200">Recent Conversations</div>
            <div className="max-h-64 overflow-y-auto px-4 py-3">
              {conversations.length === 0 ? (
                <div className="text-xs italic text-slate-500">No conversations yet</div>
              ) : (
                conversations.map(conv => (
                  <button
                    key={conv.id}
                    onClick={() => loadConversation(conv.id!)}
                    className="mb-2 w-full rounded-2xl border border-slate-800 bg-slate-900/90 px-4 py-3 text-left transition hover:border-blue-500"
                  >
                    <div className="text-sm text-slate-100 truncate">{conv.title}</div>
                    <div className="text-xs text-slate-500">{new Date(conv.updated_at).toLocaleDateString()}</div>
                  </button>
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {showConversationList && <div className="fixed inset-0 z-10" onClick={() => setShowConversationList(false)} />}
    </div>
  );
};

export default ChatContainer;
