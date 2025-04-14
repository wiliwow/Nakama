import { useState, useRef, useEffect } from "react";

interface Message {
  id: string;
  content: string;
  isUser: boolean;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputText, setInputText] = useState("");
  const [isTyping, setIsTyping] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-scroll to bottom and auto-resize textarea
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    if (textareaRef.current) {
      textareaRef.current.style.height = "48px";
      textareaRef.current.style.height = `${Math.min(
        textareaRef.current.scrollHeight,
        200
      )}px`;
    }
  }, [messages, inputText]);

  // Simulate AI response
  useEffect(() => {
    if (messages.length > 0 && messages[messages.length - 1].isUser) {
      setIsTyping(true);
      const timer = setTimeout(() => {
        setMessages((prev) => [
          ...prev,
          {
            id: Date.now().toString(),
            content: "This is a simulated AI response. Thank you for your message!",
            isUser: false,
          },
        ]);
        setIsTyping(false);
      }, 1500);
      return () => clearTimeout(timer);
    }
  }, [messages]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (inputText.trim()) {
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now().toString(),
          content: inputText,
          isUser: true,
        },
      ]);
      setInputText("");
    }
  };

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInputText(e.target.value);
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900">
      {/* Header */}
      <header className="sticky top-0 z-20 bg-gray-900 border-b border-gray-800 px-4 sm:px-6 py-4">
        <div className="max-w-4xl mx-auto flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <div className="h-8 w-8 rounded-full bg-gradient-to-br from-purple-600 to-blue-500 flex items-center justify-center">
              <span className="text-sm font-semibold text-white">AI</span>
            </div>
            <h1 className="text-lg font-semibold text-gray-200">Nakama Assistant</h1>
          </div>
          <button className="p-2 hover:bg-gray-800 rounded-full transition-colors">
            <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        </div>
      </header>

      {/* Chat Messages */}
      <main className="flex-1 overflow-y-auto px-4 py-6">
        <div className="max-w-3xl mx-auto space-y-6">
          {messages.map((msg) => (
            <div
              key={msg.id}
              className={`flex ${msg.isUser ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-[85%] lg:max-w-[75%] p-4 rounded-2xl ${
                  msg.isUser
                    ? "bg-purple-600 rounded-br-none"
                    : "bg-gray-800 rounded-bl-none"
                }`}
              >
                <div className="space-y-2">
                  <p className="text-gray-100 text-sm leading-relaxed whitespace-pre-wrap">
                    {msg.content}
                  </p>
                </div>
              </div>
            </div>
          ))}
          {isTyping && (
            <div className="flex justify-start">
              <div className="max-w-[85%] lg:max-w-[75%] p-4 rounded-2xl bg-gray-800 rounded-bl-none">
                <div className="flex space-x-2 items-center">
                  <div className="w-2 h-2 bg-gray-300 rounded-full animate-bounce" />
                  <div className="w-2 h-2 bg-gray-300 rounded-full animate-bounce delay-100" />
                  <div className="w-2 h-2 bg-gray-300 rounded-full animate-bounce delay-200" />
                </div>
              </div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>
      </main>

      {/* Input Area */}
      <footer className="sticky bottom-0 border-t border-gray-800 bg-gray-900 px-4 py-4">
        <div className="max-w-3xl mx-auto">
          <form onSubmit={handleSubmit} className="relative">
            <textarea
              ref={textareaRef}
              value={inputText}
              onChange={handleInput}
              rows={1}
              placeholder="Message Nakama AI..."
              className="w-full py-3 pl-4 pr-12 text-sm text-gray-100 bg-gray-800 rounded-2xl border border-gray-700 focus:border-purple-500 focus:ring-1 focus:ring-purple-500 resize-none scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800"
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  handleSubmit(e);
                }
              }}
            />
            <button
              type="submit"
              disabled={!inputText.trim()}
              className="absolute right-3 bottom-3 p-1.5 rounded-full bg-purple-600 hover:bg-purple-500 transition-colors disabled:opacity-50 disabled:hover:bg-purple-600"
            >
              <svg className="w-5 h-5 text-white rotate-90" viewBox="0 0 20 20" fill="currentColor">
                <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11h2v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
              </svg>
            </button>
          </form>
          <p className="text-center text-xs text-gray-500 mt-3">
            Nakama AI can make mistakes. Consider checking important information.
          </p>
        </div>
      </footer>
    </div>
  );
}

export default App;