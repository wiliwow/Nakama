import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";
import RecordingControls from "./RecordingControls";
import FileLister from "./FileLister";



const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" },
  ]);
  const [loading, setLoading] = useState(false);
  const [files, setFiles] = useState<string[]>([]);

  const handleFilesSelected = (selectedFiles: string[] | string | null) => {
    if (!selectedFiles) return;
    const incoming = Array.isArray(selectedFiles) ? selectedFiles : [selectedFiles];
    setFiles(prev => [...prev, ...incoming]);
  };

  const handleSend = async (text: string) => {
    console.log("[AI] User sent message:", text);
    setMessages(msgs => [...msgs, { sender: "user", text }]);
    // clear staged files when user sends a message
    setFiles([]);
    setLoading(true);
    // Insert a placeholder AI message that we'll update as chunks arrive
    setMessages(msgs => [...msgs, { sender: "ai", text: "" }]);

    try {
      // Start streaming on the backend; backend will emit events we listen to
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
        console.error("[AI] Full error object:", event);
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
        await unlistenChunk();
        await unlistenError();
        await unlistenDone();
      });
      await invoke("start_ai_stream", { message: text });
    } catch (err) {
      console.error("[AI] Error:", err);
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
      <div className="flex flex-col flex-1 overflow-hidden">
        <MessageList messages={messages} />
        {files && files.length > 0 && (
          <div className="px-4 pb-2">
            <div className="text-xs text-blue-300 mb-2">Linked Files:</div>
            <FileLister files={files} />
          </div>
        )}
        <MessageInput onSend={handleSend} disabled={loading} onFilesSelected={handleFilesSelected} />
      </div>
    </div>
  );
};

export default ChatContainer;

