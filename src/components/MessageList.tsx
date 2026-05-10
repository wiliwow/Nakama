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
  <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4 space-y-4 scroll-smooth">
    {messages.map((msg, idx) => (
      <div
        key={idx}
        className={`max-w-4xl mx-auto flex flex-col gap-3 rounded-3xl border p-4 shadow-lg text-sm sm:text-base animate-fade-in ${
          msg.sender === "user"
            ? "bg-blue-950/80 text-blue-100 border-slate-800/50 items-end"
            : "bg-slate-900/80 text-slate-100 border-l-4 border-amber-500/60 items-start"
        }`}
        style={{
          animationDuration: '0.2s',
          animationFillMode: 'both',
          animationTimingFunction: 'ease-out'
        }}
      >
        {msg.files && msg.files.length > 0 && (
          <div className="space-y-1 text-xs text-slate-300">
            <div className="font-semibold text-slate-100">📎 Linked Files</div>
            {msg.files.map((file, index) => (
              <div key={index} className="rounded-md bg-slate-800 px-2 py-1">{file.name}</div>
            ))}
          </div>
        )}

        {msg.screenshots && msg.screenshots.length > 0 && (
          <div className="grid gap-3 md:grid-cols-2">
            {msg.screenshots.map(shot => (
              <div key={shot.id} className="overflow-hidden rounded-2xl border border-slate-800 bg-slate-950">
                <div className="px-3 py-2 text-xs text-slate-300 bg-slate-900">{shot.label ?? `Screen ${shot.id + 1}`}</div>
                <img src={shot.dataUrl} alt={shot.label ?? `Screenshot ${shot.id}`} className="w-full object-contain" />
              </div>
            ))}
          </div>
        )}

        <div className="prose prose-invert max-w-none break-words">
          {shouldRenderMarkdown(msg.text) ? (
            <ReactMarkdown
              remarkPlugins={[remarkMath]}
              rehypePlugins={[rehypeKatex]}
              components={{
                p: ({ children }) => <p className="mb-2 last:mb-0">{children}</p>,
                h1: ({ children }) => <h1 className="text-xl font-semibold mb-2 text-amber-300">{children}</h1>,
                h2: ({ children }) => <h2 className="text-lg font-semibold mb-2 text-amber-300">{children}</h2>,
                h3: ({ children }) => <h3 className="text-base font-semibold mb-2 text-amber-300">{children}</h3>,
                ul: ({ children }) => <ul className="list-disc list-inside mb-2 space-y-1">{children}</ul>,
                ol: ({ children }) => <ol className="list-decimal list-inside mb-2 space-y-1">{children}</ol>,
                li: ({ children }) => <li className="ml-4">{children}</li>,
                code: ({ children }) => <code className="rounded bg-slate-800 px-1 py-0.5 text-sm font-mono">{children}</code>,
                pre: ({ children }) => <pre className="rounded-2xl bg-slate-900 p-3 text-sm overflow-x-auto">{children}</pre>,
                blockquote: ({ children }) => <blockquote className="border-l-4 border-slate-600 pl-4 italic text-slate-300 mb-2">{children}</blockquote>,
                strong: ({ children }) => <strong className="font-semibold text-amber-300">{children}</strong>,
                em: ({ children }) => <em className="italic text-slate-300">{children}</em>,
              }}
            >
              {msg.text}
            </ReactMarkdown>
          ) : (
            <pre className="whitespace-pre-wrap break-words text-sm">{msg.text}</pre>
          )}
        </div>
      </div>
    ))}
  </div>
);

export default MessageList;
