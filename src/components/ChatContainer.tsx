import React, { useState } from "react";
import MessageList, { Message } from "./MessageList";
import MessageInput from "./MessageInput";

const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    { sender: "ai", text: "Hi! I'm Nakama, your night-sky companion. How can I help you today?" },
  ]);
  const [loading, setLoading] = useState(false);

  const handleSend = async (text: string) => {
    setMessages(msgs => [...msgs, { sender: "user", text }]);
    setLoading(true);
    // Simulate AI response (replace with real API call)
    setTimeout(() => {
      setMessages(msgs => [
        ...msgs,
        { sender: "ai", text: `You said: "${text}". (This is a placeholder response!)` },
      ]);
      setLoading(false);
    }, 1200);
  };

  return (
    <div className="relative w-full max-w-2xl mx-auto flex flex-col h-[70vh] bg-[#181e36]/80 rounded-2xl shadow-2xl border border-blue-900 backdrop-blur-md">
      <MessageList messages={messages} />
      <MessageInput onSend={handleSend} disabled={loading} />
    </div>
  );
};

export default ChatContainer;
