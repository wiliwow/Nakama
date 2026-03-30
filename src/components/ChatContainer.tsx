import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";
import RecordingControls from "./RecordingControls";
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

  // Conversation state
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
        {
          sender: "ai",
          text: "I can now see your primary screen.",
          screenshots: [capture],
        },
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
        {
          sender: "ai",
          text: "I can now see all your screens.",
          screenshots: captures,
        },
      ]);
    } catch (err) {
      console.error("Failed to capture all screens:", err);
      setScreenCaptureStatus(`Capture failed: ${String(err)}`);
    }
  };

  // Conversation management functions
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
        // Convert conversation messages to UI messages
        const uiMessages: Message[] = conv.messages.map(msg => ({
          sender: msg.role === "user" ? "user" : "ai",
          text: msg.content,
        }));
        setMessages(uiMessages);
        setShowConversationList(false);
      }
    } catch (err) {
      console.error("Failed to load conversation:", err);
    }
  };

  const saveCurrentConversation = async () => {
    if (!currentConversation) return;

    try {
      // Convert UI messages to conversation messages
      const convMessages: ConversationMessage[] = messages
        .filter(msg => msg.sender !== "ai" || msg.text !== "Hi! I'm Nakama, your night-sky companion. How can I help you today?")
        .map(msg => ({
          role: msg.sender,
          content: msg.text,
          timestamp: new Date().toISOString(),
        }));

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

  // Load conversations on component mount
  useEffect(() => {
    loadConversations();
  }, []);

  // Index staged files immediately
  const indexStagedFiles = async () => {
    if (files.length === 0) return;

    try {
      setIndexingFiles(true);
      let successCount = 0;

      for (const file of files) {
        try {
          console.log(`[RAG] Indexing file: ${file.name}`);
          const result = await invoke<string>("rag_add_file", {
            filename: file.name,
            content: file.content,
          });
          console.log(`[RAG] Successfully indexed: ${result}`);
          successCount++;
        } catch (err) {
          console.error(`[RAG] Failed to index ${file.name}:`, err);
        }
      }

      setIndexedFilesCount(prev => prev + successCount);
      setMessages(msgs => [
        ...msgs,
        {
          sender: "ai",
          text: `✓ Indexed ${successCount}/${files.length} file(s) to memory. You can now ask questions about them!`,
        },
      ]);
      setFiles([]); // Clear staged files
    } catch (err) {
      console.error("[RAG] Error during indexing:", err);
      setMessages(msgs => [
        ...msgs,
        {
          sender: "ai",
          text: `⚠️ Error indexing files: ${String(err)}. Make sure the embedding engine is running.`,
        },
      ]);
    } finally {
      setIndexingFiles(false);
    }
  };

  const handleSend = async (text: string) => {
    console.log("[AI] User sent message:", text);

    // Ensure we have a current conversation
    if (!currentConversation) {
      await createNewConversation();
    }

    // Add user message with attached files
    setMessages(msgs => [...msgs, { sender: "user", text, files: files.length > 0 ? files : undefined }]);
    setLoading(true);

    // Insert a placeholder AI message that we'll update as chunks arrive
    setMessages(msgs => [...msgs, { sender: "ai", text: "" }]);

    try {
      // Build conversation history for context
      const conversationHistory: ConversationMessage[] = messages
        .filter(msg => msg.sender !== "ai" || msg.text !== "Hi! I'm Nakama, your night-sky companion. How can I help you today?")
        .map(msg => ({
          role: msg.sender,
          content: msg.text,
          timestamp: new Date().toISOString(),
        }));

      if (screenVisionEnabled && screenCaptures.length > 0) {
        conversationHistory.push({
          role: "system",
          content: "Screen vision is enabled and the latest display capture is available in the UI.",
          timestamp: new Date().toISOString(),
        });
      }

      // If we have indexed files (indexedFilesCount > 0), try to retrieve context
      if (indexedFilesCount > 0) {
        console.log("[AI] Attempting to retrieve context from indexed memory...");
        try {
          const retrieved = await invoke<RetrievedPassage[]>(
            "rag_retrieve",
            { query_text: text, top_k: 10 }
          );

          if (retrieved && retrieved.length > 0) {
            const retrievedContext = retrieved
              .map((r, idx) => {
                const src = r.source ? `Source: ${r.source}` : `Source ${idx + 1}`;
                return `[${src} - Relevance: ${(r.score * 100).toFixed(1)}%]\n${r.content}`;
              })
              .join("\n\n---\n\n");

            // Add retrieved context to the conversation history for the AI
            conversationHistory.push({
              role: "system",
              content: `Retrieved context from indexed files:\n\n${retrievedContext}`,
              timestamp: new Date().toISOString(),
            });
            console.log("[AI] Added retrieval context to conversation history");
          }
        } catch (ragErr) {
          console.warn("[AI] RAG retrieval failed (continuing without indexed context):", ragErr);
        }
      }

      // If staged files are attached (not yet indexed), index them first
      if (files.length > 0) {
        console.log("[AI] Indexing staged files before querying...");
        await indexStagedFiles();
      }

      // Start streaming with conversation context
      const unlistenChunk = await listen<string>("ai-stream-chunk", (event) => {
        console.log("[AI] Received chunk:", event.payload);
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
        console.error("[AI] Stream error event:", event.payload);
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
        console.log("[AI] Stream completed");
        setLoading(false);
        await saveCurrentConversation(); // Save after AI response
        await unlistenChunk();
        await unlistenError();
        await unlistenDone();
      });

      // Trigger streaming with conversation context
      console.log("[AI] Invoking ask_ai_stream_with_conversation");
      await invoke("ask_ai_stream_with_conversation", {
        prompt: text,
        conversationHistory
      });
      console.log("[AI] ask_ai_stream_with_conversation invoked successfully");
    } catch (err) {
      console.error("[AI] Exception caught in handleSend:", err);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `Sorry, I encountered an error: ${String(err)}. Make sure Ollama is running locally (ollama run deepseek).` },
      ]);
      setLoading(false);
    }
  };

  return (
    <div className="relative w-full max-w-2xl mx-auto flex flex-col h-[70vh] gap-4 bg-[#181e36]/80 rounded-2xl shadow-2xl border border-blue-900 backdrop-blur-md">
      <RecordingControls />

      {/* Conversation Management Header */}
      <div className="px-4 pt-2 flex items-center justify-between border-b border-blue-800/50 pb-2">
        <div className="flex items-center gap-2">
          <button
            onClick={createNewConversation}
            className="text-xs px-3 py-1 rounded bg-blue-700 hover:bg-blue-600 text-blue-100 transition-all"
            title="New Conversation"
          >
            ➕ New
          </button>
          <button
            onClick={() => setShowConversationList(!showConversationList)}
            className="text-xs px-3 py-1 rounded bg-purple-700 hover:bg-purple-600 text-purple-100 transition-all"
            title="Load Conversation"
          >
            📂 Load
          </button>
        </div>
        {currentConversation && (
          <div className="text-xs text-blue-300 truncate max-w-xs">
            {currentConversation.title}
          </div>
        )}
      </div>

      <div className="px-4 grid gap-4 md:grid-cols-[1.1fr_380px]">
        <div>
          <ScreenVisionPanel
            enabled={screenVisionEnabled}
            captures={screenCaptures}
            onToggleEnabled={() => setScreenVisionEnabled(prev => !prev)}
            onCapturePrimary={capturePrimaryScreen}
            onCaptureAll={captureAllScreens}
          />
          {screenCaptureStatus && (
            <div className="mt-2 text-xs text-blue-300">{screenCaptureStatus}</div>
          )}
        </div>
        <AutomationControlPanel />
      </div>

      {/* Conversation List Modal */}
      {showConversationList && (
        <div
          className="absolute top-full mt-2 left-0 right-0 bg-[#1a1f35]/95 rounded-lg border border-blue-800/50 shadow-xl z-10 max-h-64 overflow-y-auto"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="p-3">
            <div className="text-sm text-blue-200 mb-2 font-semibold">Recent Conversations</div>
            {conversations.length === 0 ? (
              <div className="text-xs text-blue-400 italic">No conversations yet</div>
            ) : (
              conversations.map((conv) => (
                <button
                  key={conv.id}
                  onClick={() => loadConversation(conv.id!)}
                  className="w-full text-left p-2 rounded hover:bg-blue-800/30 transition-all mb-1"
                >
                  <div className="text-sm text-blue-100 truncate">{conv.title}</div>
                  <div className="text-xs text-blue-400">
                    {new Date(conv.updated_at).toLocaleDateString()}
                  </div>
                </button>
              ))
            )}
          </div>
        </div>
      )}

      {/* Click outside to close conversation list */}
      {showConversationList && (
        <div
          className="fixed inset-0 z-0"
          onClick={() => setShowConversationList(false)}
        />
      )}

      <div className="flex flex-col flex-1 overflow-hidden">
        <MessageList messages={messages} />
        {files && files.length > 0 && (
          <div className="px-4 pb-2">
            <div className="text-xs text-blue-300 mb-2">Staged Files ({files.length}):</div>
            <div className="flex flex-wrap gap-2 mb-2">
              {files.map((f, i) => (
                <div key={i} className="text-xs bg-blue-700 px-2 py-1 rounded text-blue-100">
                  📄 {f.name}
                </div>
              ))}
            </div>
            <button
              onClick={indexStagedFiles}
              disabled={indexingFiles || loading}
              className="text-xs px-3 py-1 rounded bg-green-700 hover:bg-green-600 text-green-100 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
            >
              {indexingFiles ? "Indexing..." : "✓ Index Files"}
            </button>
          </div>
        )}
        {indexedFilesCount > 0 && (
          <div className="px-4 pb-2 text-xs text-green-300">
            📚 {indexedFilesCount} file(s) in memory
          </div>
        )}
        <MessageInput onSend={handleSend} disabled={loading || indexingFiles} onFilesSelected={handleFilesSelected} />
      </div>
    </div>
  );
};

export default ChatContainer;

