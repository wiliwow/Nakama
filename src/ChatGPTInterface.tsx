import React, { useState } from 'react';

interface Message {
    id: number;
    sender: 'user' | 'bot';
    text: string;
}

const ChatGPTInterface = () => {
    const [messages, setMessages] = useState<Message[]>([]);
    const [input, setInput] = useState('');

    const sendMessage = () => {
        if (input.trim() === '') return;

        const userMessage: Message = {
            id: Date.now(),
            sender: 'user',
            text: input,
        };

        // Simulate a bot reply
        const botMessage: Message = {
            id: Date.now() + 1,
            sender: 'bot',
            text: `Bot: I'm a bot response to "${input}"`,
        };

        setMessages((prev) => [...prev, userMessage, botMessage]);
        setInput('');
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    };

    return (
        <div className="flex flex-col items-center min-h-screen bg-gradient-to-br from-indigo-100 via-purple-50 to-blue-100 p-4">
            {/* Header */}
            <header className="w-full max-w-3xl mb-6">
                <h1 className="text-3xl font-bold text-center text-gray-800 drop-shadow-sm">
                    ChatGPT Interface
                </h1>
            </header>
            
            <div className="flex flex-col w-full max-w-3xl bg-white/80 backdrop-blur-sm shadow-2xl rounded-2xl overflow-hidden border border-gray-200">
                {/* Messages Container */}
                <div className="flex-1 min-h-[500px] p-6 overflow-y-auto space-y-6 scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-transparent">
                    {messages.map((message) => (
                        <div
                            key={message.id}
                            className={`flex items-end gap-2 ${
                                message.sender === 'user' ? 'justify-end' : 'justify-start'
                            }`}
                        >
                            {message.sender === 'bot' && (
                                <div className="w-8 h-8 rounded-full bg-gradient-to-r from-purple-500 to-indigo-500 flex items-center justify-center text-white text-sm">
                                    AI
                                </div>
                            )}
                            <div
                                className={`max-w-[80%] px-4 py-3 rounded-2xl shadow-sm ${
                                    message.sender === 'user'
                                        ? 'bg-gradient-to-r from-blue-500 to-indigo-600 text-white'
                                        : 'bg-gray-100 text-gray-800'
                                } transform transition-all duration-200 hover:scale-[1.02]`}
                            >
                                <p className="text-sm md:text-base">{message.text}</p>
                            </div>
                            {message.sender === 'user' && (
                                <div className="w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-indigo-500 flex items-center justify-center text-white text-sm">
                                    You
                                </div>
                            )}
                        </div>
                    ))}
                </div>

                {/* Input Container */}
                <div className="p-4 bg-gray-50 border-t border-gray-200">
                    <div className="relative flex items-center">
                        <input
                            type="text"
                            className="w-full px-4 py-3 pr-12 bg-white border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent shadow-sm text-gray-800 placeholder-gray-400"
                            placeholder="Type your message..."
                            value={input}
                            onChange={(e) => setInput(e.target.value)}
                            onKeyDown={handleKeyDown}
                        />
                        <button
                            onClick={sendMessage}
                            className="absolute right-2 p-2 text-blue-500 hover:text-blue-600 focus:outline-none"
                        >
                            <svg
                                className="w-6 h-6"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth={2}
                                    d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                                />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default ChatGPTInterface;