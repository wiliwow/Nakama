import React, { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList from "./MessageList";
import MessageInput from "./MessageInput";
import ChatHeader from "./ChatHeader";
import FileStager from "./FileStager";
import SidePanel from "./SidePanel";
import ConversationList from "./ConversationList";
import MemoryPanel from "./MemoryPanel";
import { useToast } from "../contexts/ToastContext";
import { useMemoryRecall } from "../hooks/useMemoryRecall";
import { useFileIndexer } from "../hooks/useFileIndexer";
import {
  Conversation,
  ConversationMessage,
  RetrievedPassage,
  ScreenCapture,
  Message,
} from "../types";

const GREETING = "Hi! I'm Nakama, your night-sky companion. How can I help you today?";
const isGreeting = (msg: Message) =>
  msg.sender === "ai" && msg.text === GREETING;

function buildHistory(messages: Message[]): ConversationMessage[] {
  return messages
    .filter((msg) => !(msg.sender === "ai" && isGreeting(msg)))
    .map((msg) => ({
      role: msg.sender === "ai" ? "assistant" : msg.sender,
      content: msg.text,
      timestamp: msg.timestamp ?? new Date().toISOString(),
    }));
}

const ChatContainer: React.FC = () => {
  const { showToast } = useToast();

  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: GREETING },
  ]);
  const [loading, setLoading] = useState(false);
  const [files, setFiles] = useState<{ name: string; content: string }[]>([]);
  const fileIndexer = useFileIndexer();

  const [screenVisionEnabled, setScreenVisionEnabled] = useState(false);
  const [screenCaptures, setScreenCaptures] = useState<ScreenCapture[]>([]);
  const [screenCaptureLoading, setScreenCaptureLoading] = useState<
    "primary" | "all" | null
  >(null);

  const [currentConversation, setCurrentConversation] =
    useState<Conversation | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [showConversationList, setShowConversationList] = useState(false);

  const [isListening, setIsListening] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [voiceConversationEnabled, setVoiceConversationEnabled] =
    useState(false);

  const [showMemoryPanel, setShowMemoryPanel] = useState(false);
  const [memoryCount, setMemoryCount] = useState({
    episodes: 0,
    facts: 0,
    goals: 0,
  });
  const [localMemoryAvailable, setLocalMemoryAvailable] = useState<boolean | null>(null);

  const { recall, remember } = useMemoryRecall({
    localMemoryAvailable: localMemoryAvailable ?? false,
    memoryCount,
  });

  const messagesRef = useRef<Message[]>(messages);
  const isStreamingRef = useRef(false);

  useEffect(() => {
    messagesRef.current = messages;
  }, [messages]);

  useEffect(() => {
    const initMemory = async () => {
      try {
        const summary = await invoke<{
          total_episodes: number;
          total_facts: number;
          total_goals: number;
        }>("memory_init");
        setLocalMemoryAvailable(true);
        setMemoryCount({
          episodes: summary.total_episodes || 0,
          facts: summary.total_facts || 0,
          goals: summary.total_goals || 0,
        });
      } catch (err) {
        console.error("[Memory] memory_init failed:", err);
        setLocalMemoryAvailable(false);
      }
    };
    initMemory();
  }, []);

  const saveDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!currentConversation || messages.length === 0) return;
    if (saveDebounceRef.current) clearTimeout(saveDebounceRef.current);

    saveDebounceRef.current = setTimeout(async () => {
      try {
        const convMessages: ConversationMessage[] = messagesRef.current
          .filter((msg) => !(msg.sender === "ai" && isGreeting(msg)))
          .map((msg) => ({
            role: msg.sender === "ai" ? "assistant" : msg.sender,
            content: msg.text,
            timestamp: msg.timestamp ?? new Date().toISOString(),
          }));

        await invoke("conversation_save", {
          conversation: {
            ...currentConversation,
            messages: convMessages,
            updated_at: new Date().toISOString(),
          },
        });
      } catch (err) {
        console.warn("[AutoSave] conversation_save failed:", err);
      }
    }, 30_000);

    return () => {
      if (saveDebounceRef.current) clearTimeout(saveDebounceRef.current);
    };
  }, [messages, currentConversation]);

  const handleFilesSelected = useCallback(
    (selectedFiles: { name: string; content: string }[] | null) => {
      if (!selectedFiles) return;
      setFiles((prev) => [...prev, ...selectedFiles]);
    },
    []
  );

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const capturePrimaryScreen = useCallback(async () => {
    try {
      setScreenCaptureLoading("primary");
      const capture = await invoke<any>("capture_primary_screen");
      setScreenCaptures([capture]);
      setMessages((msgs) => [
        ...msgs,
        {
          sender: "ai",
          text: "I can now see your primary screen.",
          screenshots: [capture],
        },
      ]);
    } catch (err) {
      console.error("Failed to capture primary screen:", err);
      showToast(`Capture failed: ${String(err)}`, { type: "error" });
      setMessages((msgs) => [
        ...msgs,
        { sender: "ai", text: `Capture failed: ${String(err)}` },
      ]);
    } finally {
      setScreenCaptureLoading(null);
    }
  }, [showToast]);

  const captureAllScreens = useCallback(async () => {
    try {
      setScreenCaptureLoading("all");
      const captures = await invoke<any[]>("capture_all_screens");
      setScreenCaptures(captures);
      setMessages((msgs) => [
        ...msgs,
        {
          sender: "ai",
          text: "I can now see all your screens.",
          screenshots: captures,
        },
      ]);
    } catch (err) {
      console.error("Failed to capture all screens:", err);
      showToast(`Capture failed: ${String(err)}`, { type: "error" });
      setMessages((msgs) => [
        ...msgs,
        { sender: "ai", text: `Capture failed: ${String(err)}` },
      ]);
    } finally {
      setScreenCaptureLoading(null);
    }
  }, [showToast]);

  const loadConversations = useCallback(async () => {
    try {
      const convs = await invoke<Conversation[]>("conversation_list");
      setConversations(convs);
    } catch (err) {
      console.error("[Conversation] conversation_list failed:", err);
      showToast("Failed to load conversations", { type: "error" });
    }
  }, [showToast]);

  const createNewConversation = useCallback(async () => {
    try {
      const title = `Conversation ${new Date().toLocaleString()}`;
      const conv = await invoke<Conversation>("conversation_create", { title });
      setCurrentConversation(conv);
      setMessages([{ sender: "ai", text: GREETING }]);
      setMemoryCount((prev) => ({ ...prev, episodes: 0, facts: 0, goals: 0 }));
      await loadConversations();
      return conv;
    } catch (err: unknown) {
      showToast(`Failed to create conversation: ${err}`, { type: "error" });
      return null;
    }
  }, [loadConversations, showToast]);

  const loadConversation = useCallback(
    async (id: number) => {
      try {
        const conv = await invoke<Conversation | null>("conversation_load", {
          id,
        });
        if (conv) {
          setCurrentConversation(conv);
          setMessages(
            conv.messages.map((msg) => ({
              sender: msg.role === "user" ? "user" : "ai",
              text: msg.content,
              timestamp: msg.timestamp,
            }))
          );
          setShowConversationList(false);
        }
      } catch (err) {
        console.error("[Conversation] conversation_load failed:", err);
        showToast("Failed to load conversation", { type: "error" });
      }
    },
    [showToast]
  );

  const saveCurrentConversationExplicit = useCallback(async () => {
    if (!currentConversation) return;

    try {
      const convMessages: ConversationMessage[] = messages
        .filter((msg) => !(msg.sender === "ai" && isGreeting(msg)))
        .map((msg) => ({
          role: msg.sender === "ai" ? "assistant" : msg.sender,
          content: msg.text,
          timestamp: msg.timestamp ?? new Date().toISOString(),
        }));

      await invoke("conversation_save", {
        conversation: {
          ...currentConversation,
          messages: convMessages,
          updated_at: new Date().toISOString(),
        },
      });
      showToast("Conversation saved", { type: "success", duration: 2000 });
    } catch (err) {
      console.error("[Conversation] saveCurrentConversationExplicit failed:", err);
      showToast("Failed to save conversation", { type: "error" });
    }
  }, [messages, currentConversation, showToast]);

  const handleStreamChunk = useCallback((chunk: string) => {
    setMessages((msgs) => {
      const copy = [...msgs];
      const lastIdx = copy.length - 1;
      if (lastIdx >= 0 && copy[lastIdx].sender === "ai") {
        copy[lastIdx] = {
          ...copy[lastIdx],
          text: (copy[lastIdx].text ?? "") + (chunk ?? ""),
        };
      }
      return copy;
    });
  }, []);

  const handleStreamError = useCallback(
    (err: string) => {
      setMessages((msgs) => {
        const copy = [...msgs];
        const lastIdx = copy.length - 1;
        if (lastIdx >= 0 && copy[lastIdx].sender === "ai") {
          copy[lastIdx] = { ...copy[lastIdx], text: `Error: ${err}` };
        }
        return copy;
      });
      showToast(`AI error: ${err}`, { type: "error" });
    },
    [showToast]
  );

  const handleStreamDone = useCallback(async () => {
    setLoading(false);
    isStreamingRef.current = false;
    try {
      await saveCurrentConversationExplicit();
    } catch (err) {
      console.warn("[StreamDone] auto-save after stream failed:", err);
    }
  }, [saveCurrentConversationExplicit]);

  const handleSend = useCallback(
    async (text: string) => {
      if (!text.trim() || loading) return;
      const userMessage = text.trim();

      setMessages((msgs) => [...msgs, { sender: "user", text: userMessage }]);
      setLoading(true);
      isStreamingRef.current = true;

      if (!currentConversation) {
        const conv = await createNewConversation();
        if (!conv) {
          setLoading(false);
          isStreamingRef.current = false;
          return;
        }
      }

      try {
        await remember("user", userMessage);

        const currentMessages = messagesRef.current;
        const history = buildHistory(currentMessages);

        const userWantsMemory =
          userMessage.toLowerCase().includes("remember") ||
          userMessage.toLowerCase().includes("recall") ||
          userMessage.toLowerCase().includes("memory") ||
          userMessage.toLowerCase().includes("what do you know") ||
          userMessage.toLowerCase().includes("what have we");

        if (localMemoryAvailable || userWantsMemory) {
          const { text: recalledText, relevant } = await recall(userMessage);
          if (relevant) {
            history.push({
              role: "system",
              content: `From persistent memory:\n\n${recalledText}`,
              timestamp: new Date().toISOString(),
            });
          }
        }

        if (screenVisionEnabled && screenCaptures.length > 0) {
          history.push({
            role: "system",
            content:
              "Screen vision is enabled and the latest display capture is available in the UI.",
            timestamp: new Date().toISOString(),
          });
        }

        if (fileIndexer.indexedCount > 0) {
          try {
            const retrieved = await invoke<RetrievedPassage[]>(
              "rag_retrieve",
              { query_text: userMessage, top_k: 10 }
            );
            if (retrieved && retrieved.length > 0) {
              history.push({
                role: "system",
                content: `Retrieved context from indexed files:\n\n${retrieved
                  .map(
                    (r, idx) => {
                      const src = r.source
                        ? `Source: ${r.source}`
                        : `Source ${idx + 1}`;
                      return `[${src} - Relevance: ${(r.score * 100).toFixed(
                        1
                      )}%]\n${r.content}`;
                    }
                  )
                  .join("\n\n---\n\n")}`,
                timestamp: new Date().toISOString(),
              });
            }
          } catch (err) {
            console.warn("[RAG] rag_retrieve failed — continuing without RAG context:", err);
          }
        }

        if (files.length > 0) {
          await fileIndexer.indexFiles(files);
          setFiles([]);
        }

        let prompt = userMessage;
        if (localMemoryAvailable) {
          try {
            prompt =
              (await invoke<string>("memory_build_prompt", {
                userPrompt: userMessage,
              })) || userMessage;
          } catch (err) {
            console.warn("[Memory] memory_build_prompt failed — using raw user prompt:", err);
          }
        }

        const unlistenChunk = await listen<string>(
          "ai-stream-chunk",
          (event) => {
            handleStreamChunk(event.payload ?? "");
          }
        );

        const unlistenError = await listen<string>(
          "ai-stream-error",
          (event) => {
            handleStreamError(event.payload ?? "Unknown error");
          }
        );

        await listen<void>("ai-stream-done", async () => {
          await handleStreamDone();
          unlistenChunk();
          unlistenError();
        });

        const imageB64 =
          screenVisionEnabled && screenCaptures.length > 0
            ? screenCaptures[screenCaptures.length - 1].dataUrl
            : undefined;

        await invoke("execute_ai_with_actions", {
          prompt,
          conversationHistory: history,
          imageB64,
        });
      } catch (err) {
        console.error("[AI] execute_ai_with_actions failed:", err);
        showToast(
          "Error: AI execution failed. Configure LLM_ENDPOINT and LLM_MODEL.",
          { type: "error" }
        );
        setMessages((msgs) => [
          ...msgs,
          {
            sender: "ai",
            text: "Sorry, I encountered an error. Configure LLM_ENDPOINT and LLM_MODEL environment variables.",
          },
        ]);
        setLoading(false);
        isStreamingRef.current = false;
      }
    },
    [
      loading,
      currentConversation,
      createNewConversation,
      remember,
      recall,
      localMemoryAvailable,
      screenVisionEnabled,
      screenCaptures,
      fileIndexer.indexedCount,
      fileIndexer.indexFiles,
      files,
      handleStreamChunk,
      handleStreamError,
      handleStreamDone,
      showToast,
    ]
  );

  useEffect(() => {
    loadConversations();
  }, [loadConversations]);

  return (
    <div className="relative h-screen flex flex-col bg-slate-950 text-slate-100 overflow-hidden">
      <div className="flex-1 mx-auto w-full max-w-[1500px] p-4 min-h-0 flex flex-col">
        <div className="flex-1 flex flex-col min-h-0 rounded-[32px] border border-slate-800/60 bg-slate-950/95 shadow-2xl overflow-hidden">
          <ChatHeader
            indexedFilesCount={fileIndexer.indexedCount}
            voiceConversationEnabled={voiceConversationEnabled}
            onNewConversation={createNewConversation}
            onToggleConversationList={() =>
              setShowConversationList((prev) => !prev)
            }
            onToggleVoiceChat={setVoiceConversationEnabled}
            currentConversationTitle={currentConversation?.title}
            onToggleMemory={() => setShowMemoryPanel((prev) => !prev)}
            memoryCount={memoryCount}
            localMemoryAvailable={localMemoryAvailable ?? false}
          />

          <div className="flex-1 flex flex-col md:flex-row min-h-0">
            <div className="flex-1 flex flex-col min-h-0">
              <MessageList messages={messages} />

              <FileStager
                files={files}
                indexingFiles={fileIndexer.indexing}
                onIndexFiles={fileIndexer.indexFiles}
                onRemoveFile={removeFile}
              />

              <div className="border-t border-slate-800/50">
                <MessageInput
                  onSend={handleSend}
                  disabled={loading || fileIndexer.indexing}
                  onFilesSelected={handleFilesSelected}
                  isListening={isListening}
                  isSpeaking={isSpeaking}
                  onVoiceInput={async (t: string) => {
                    setIsListening(false);
                    if (t.trim()) await handleSend(t);
                  }}
                  onVoiceOutput={async () => {
                    setIsSpeaking(true);
                  }}
                />
              </div>
            </div>

            <SidePanel
              screenVisionEnabled={screenVisionEnabled}
              screenCaptures={screenCaptures}
              screenCaptureLoading={screenCaptureLoading}
              onScreenVisionToggle={() =>
                setScreenVisionEnabled((prev) => !prev)
              }
              onCapturePrimary={capturePrimaryScreen}
              onCaptureAll={captureAllScreens}
            />
          </div>
        </div>
      </div>

      {showConversationList && (
        <ConversationList
          conversations={conversations}
          onSelect={loadConversation}
          onClose={() => setShowConversationList(false)}
        />
      )}

      {showMemoryPanel && (
        <MemoryPanel
          isOpen={showMemoryPanel}
          onClose={() => setShowMemoryPanel(false)}
        />
      )}
    </div>
  );
};

export default ChatContainer;
