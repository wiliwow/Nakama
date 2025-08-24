import React from "react";

export interface Message {
  sender: "user" | "ai";
  text: string;
}

const MessageList: React.FC<{ messages: Message[] }> = ({ messages }) => (
  <div className="flex-1 overflow-y-auto px-4 py-2 space-y-4">
    {messages.map((msg, idx) => (
      <div
        key={idx}
        className={`max-w-xl mx-auto px-4 py-2 rounded-lg shadow-md text-base whitespace-pre-line ${
          msg.sender === "user"
            ? "bg-blue-900 text-blue-100 self-end ml-auto"
            : "bg-gray-800 text-gray-100 self-start mr-auto border-l-4 border-yellow-300"
        }`}
      >
        {msg.text}
      </div>
    ))}
  </div>
);

export default MessageList;
