import React from "react";
import ReactMarkdown from 'react-markdown';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import 'katex/dist/katex.min.css';

export interface Message {
  sender: "user" | "ai";
  text: string;
  files?: { name: string; content: string }[];
  screenshots?: { id: number; dataUrl: string; label?: string }[];
}

const shouldRenderMarkdown = (text: string) => {
  const markdownPattern = /(```[\s\S]*?```|`[^`\n]+`|\$\$[\s\S]*?\$\$|\$[^$\n]+\$)/;
  return markdownPattern.test(text);
};

const MessageList: React.FC<{ messages: Message[] }> = ({ messages }) => (
  <div className="flex-1 overflow-y-auto px-4 py-2 space-y-4">
    {messages.map((msg, idx) => (
      <div
        key={idx}
        className={`max-w-4xl mx-auto px-4 py-2 rounded-lg shadow-md text-base ${
          msg.sender === "user"
            ? "bg-blue-900 text-blue-100 self-end ml-auto"
            : "bg-gray-800 text-gray-100 self-start mr-auto border-l-4 border-yellow-300"
        }`}
      >
        {msg.files && msg.files.length > 0 && (
          <div className="mb-2 text-xs text-blue-300 font-semibold">
            📎 Linked Files:
            {msg.files.map((f, i) => (
              <div key={i} className="mt-1 text-blue-200">{f.name}</div>
            ))}
          </div>
        )}
        {msg.screenshots && msg.screenshots.length > 0 && (
          <div className="mb-2 grid grid-cols-1 md:grid-cols-2 gap-2">
            {msg.screenshots.map((shot) => (
              <div key={shot.id} className="rounded-lg overflow-hidden border border-blue-800 bg-black">
                <div className="px-2 py-1 text-xs text-blue-200 bg-[#111827]">{shot.label ?? `Screen ${shot.id + 1}`}</div>
                <img src={shot.dataUrl} alt={shot.label ?? `Screenshot ${shot.id}`} className="w-full h-auto object-contain" />
              </div>
            ))}
          </div>
        )}
        <div className="prose prose-invert max-w-none">
          {shouldRenderMarkdown(msg.text) ? (
            <ReactMarkdown
              remarkPlugins={[remarkMath]}
              rehypePlugins={[rehypeKatex]}
              components={{
                p: ({ children }) => <p className="mb-2 last:mb-0">{children}</p>,
                h1: ({ children }) => <h1 className="text-xl font-bold mb-2 text-yellow-300">{children}</h1>,
                h2: ({ children }) => <h2 className="text-lg font-bold mb-2 text-yellow-300">{children}</h2>,
                h3: ({ children }) => <h3 className="text-md font-bold mb-2 text-yellow-300">{children}</h3>,
                ul: ({ children }) => <ul className="list-disc list-inside mb-2 space-y-1">{children}</ul>,
                ol: ({ children }) => <ol className="list-decimal list-inside mb-2 space-y-1">{children}</ol>,
                li: ({ children }) => <li className="ml-4">{children}</li>,
                code: ({ children }) => <code className="bg-gray-700 px-1 py-0.5 rounded text-sm font-mono">{children}</code>,
                pre: ({ children }) => <pre className="bg-gray-700 p-3 rounded mb-2 overflow-x-auto">{children}</pre>,
                blockquote: ({ children }) => <blockquote className="border-l-4 border-gray-500 pl-4 italic text-gray-300 mb-2">{children}</blockquote>,
                strong: ({ children }) => <strong className="font-bold text-yellow-200">{children}</strong>,
                em: ({ children }) => <em className="italic text-blue-200">{children}</em>,
              }}
            >
              {msg.text}
            </ReactMarkdown>
          ) : (
            <pre className="whitespace-pre-wrap break-words">{msg.text}</pre>
          )}
        </div>
      </div>
    ))}
  </div>
);

export default MessageList;
