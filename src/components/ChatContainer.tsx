import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";
import ChatHeader from "./ChatHeader";
import FileStager from "./FileStager";
import SidePanel from "./SidePanel";
import ConversationList from "./ConversationList";

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
  // Core state
  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" },
  ]);
  const [loading, setLoading] = useState(false);

  // Files & indexing
  const [files, setFiles] = useState<{ name: string; content: string }[]>([]);
  const [indexingFiles, setIndexingFiles] = useState(false);
  const [indexedFilesCount, setIndexedFilesCount] = useState(0);

  // Screen vision
  const [screenVisionEnabled, setScreenVisionEnabled] = useState(false);
  const [screenCaptures, setScreenCaptures] = useState<any[]>([]);
  const [screenCaptureLoading, setScreenCaptureLoading] = useState<"primary" | "all" | null>(null);

  // Conversations
  const [currentConversation, setCurrentConversation] = useState<Conversation | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [showConversationList, setShowConversationList] = useState(false);

  // Voice state
  const [isListening, setIsListening] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [voiceConversationEnabled, setVoiceConversationEnabled] = useState(false);

  // Event listener cleanup
  const unsubscribesRef = useRef<{ chunk?: () => void; error?: () => void; done?: () => void }>({});

  // File handling
  const handleFilesSelected = (selectedFiles: { name: string; content: string }[] | null) => {
    if (!selectedFiles) return;
    setFiles(prev => [...prev, ...selectedFiles]);
  };

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  // Screen capture
  const capturePrimaryScreen = async () => {
    try {
      setScreenCaptureLoading("primary");
      const capture = await invoke<any>("capture_primary_screen");
      setScreenCaptures([capture]);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: "I can now see your primary screen.", screenshots: [capture] },
      ]);
    } catch (err) {
      console.error("Failed to capture primary screen:", err);
      setMessages(msgs => [...msgs, { sender: "ai", text: `Capture failed: ${String(err)}` }]);
    } finally {
      setScreenCaptureLoading(null);
    }
  };

  const captureAllScreens = async () => {
    try {
      setScreenCaptureLoading("all");
      const captures = await invoke<any[]>("capture_all_screens");
      setScreenCaptures(captures);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: "I can now see all your screens.", screenshots: captures },
      ]);
    } catch (err) {
      console.error("Failed to capture all screens:", err);
      setMessages(msgs => [...msgs, { sender: "ai", text: `Capture failed: ${String(err)}` }]);
    } finally {
      setScreenCaptureLoading(null);
    }
  };

  // Conversation management
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

  // File indexing
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

  // Send message
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
      unsubscribesRef.current.chunk = unlistenChunk;

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
      unsubscribesRef.current.error = unlistenError;

      const unlistenDone = await listen<void>("ai-stream-done", async () => {
        setLoading(false);
        await saveCurrentConversation();
        unlistenChunk();
        unlistenError();
        unlistenDone();
        unsubscribesRef.current = {};
      });
      unsubscribesRef.current.done = unlistenDone;

      await invoke("execute_ai_with_actions", { prompt: text, conversationHistory: history });
    } catch (err) {
      console.error("[AI] Exception caught in handleSend:", err);
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `Sorry, I encountered an error: ${String(err)}. Make sure Ollama is running locally.` },
      ]);
      setLoading(false);
    }
  };

  // Initial load
  useEffect(() => {
    loadConversations();
  }, []);

  // Cleanup
  useEffect(() => {
    return () => {
      unsubscribesRef.current.chunk?.();
      unsubscribesRef.current.error?.();
      unsubscribesRef.current.done?.();
    };
  }, []);

  return (
    <div className="relative h-screen flex flex-col bg-slate-950 text-slate-100 overflow-hidden">
      {/* Main container with rounded corners */}
      <div className="flex-1 mx-auto w-full max-w-[1500px] p-4 min-h-0 flex flex-col">
        <div className="flex-1 flex flex-col min-h-0 rounded-[32px] border border-slate-800/60 bg-slate-950/95 shadow-2xl overflow-hidden">
          {/* Header */}
          <ChatHeader
            indexedFilesCount={indexedFilesCount}
            voiceConversationEnabled={voiceConversationEnabled}
            onNewConversation={createNewConversation}
            onToggleConversationList={() => setShowConversationList(prev => !prev)}
            onToggleVoiceChat={setVoiceConversationEnabled}
            currentConversationTitle={currentConversation?.title}
          />

          {/* Main content area */}
          <div className="flex-1 flex flex-col md:flex-row min-h-0">
            {/* Chat area */}
            <div className="flex-1 flex flex-col min-h-0">
              <div className="flex-1 overflow-y-auto px-4 py-4">
                <MessageList messages={messages} />
              </div>

              {/* File stager */}
              <FileStager
                files={files}
                indexingFiles={indexingFiles}
                onIndexFiles={indexStagedFiles}
                onRemoveFile={removeFile}
              />

              {/* Message input */}
              <div className="border-t border-slate-800/50">
                <MessageInput
                  onSend={handleSend}
                  disabled={loading || indexingFiles}
                  onFilesSelected={handleFilesSelected}
                  isListening={isListening}
                  isSpeaking={isSpeaking}
                  onVoiceInput={async (text: string) => {
                    setIsListening(false);
                    await handleSend(text);
                  }}
                  onVoiceOutput={async (_text: string) => {
                    setIsSpeaking(true);
                  }}
                />
              </div>
            </div>

            {/* Side panel */}
            <SidePanel
              screenVisionEnabled={screenVisionEnabled}
              screenCaptures={screenCaptures}
              screenCaptureLoading={screenCaptureLoading}
              onScreenVisionToggle={() => setScreenVisionEnabled(prev => !prev)}
              onCapturePrimary={capturePrimaryScreen}
              onCaptureAll={captureAllScreens}
            />
          </div>
        </div>
      </div>

      {/* Conversation list modal */}
      {showConversationList && (
        <ConversationList
          conversations={conversations}
          onSelect={loadConversation}
          onClose={() => setShowConversationList(false)}
        />
      )}
    </div>
  );
};

export default ChatContainer;
