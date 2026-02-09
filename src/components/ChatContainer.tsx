import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";
import RecordingControls from "./RecordingControls";

const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" },
  ]);
  const [loading, setLoading] = useState(false);

  const handleSend = async (text: string) => {
    console.log("[AI] User sent message:", text);
    setMessages(msgs => [...msgs, { sender: "user", text }]);
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

      // Trigger streaming; the command returns immediately after spawning
      console.log("[AI] Invoking ask_ai_stream with prompt:", text);
      await invoke("ask_ai_stream", { prompt: text });
      console.log("[AI] ask_ai_stream invoked successfully");
    } catch (err) {
      console.error("[AI] Exception caught in handleSend:", err);
      console.error("[AI] Error type:", typeof err);
      console.error("[AI] Error details:", JSON.stringify(err, null, 2));
      console.error("[AI] Error toString():", String(err));
      if (err instanceof Error) {
        console.error("[AI] Error message:", err.message);
        console.error("[AI] Error stack:", err.stack);
      }
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
        <MessageInput onSend={handleSend} disabled={loading} />
      </div>
    </div>
  );
};

export default ChatContainer;
