import React, { useRef, useEffect, useCallback } from "react";
import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import { Message } from "../types";

interface MessageListProps {
  messages: Message[];
}

const shouldRenderMarkdown = (text: string): boolean => {
  const markdownPattern = /(```[\s\S]*?```|`[^`\n]+`|\$\$[\s\S]*?\$\$|\$[^$\n]+\$)/;
  return markdownPattern.test(text);
};

const MessageList: React.FC<MessageListProps> = ({ messages }) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const observerRef = useRef<MutationObserver | null>(null);

  const scrollToBottom = useCallback(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  // Auto-scroll on new messages
  useEffect(() => {
    scrollToBottom();
  }, [messages.length, scrollToBottom]);

  // Observe DOM changes for streaming chunks (smooth scroll)
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;

    const observer = new MutationObserver(() => {
      // Check if user is near the bottom (within 100px)
      const distanceFromBottom =
        container.scrollHeight - container.scrollTop - container.clientHeight;
      if (distanceFromBottom < 100) {
        scrollToBottom();
      }
    });

    observer.observe(container, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    observerRef.current = observer;

    return () => {
      observer.disconnect();
    };
  }, [scrollToBottom]);

  return (
    <div
      ref={scrollRef}
      className="min-h-0 flex-1 overflow-y-auto px-4 py-4 space-y-4 scroll-smooth"
    >
      {messages.length === 0 ? (
        <div className="flex items-center justify-center h-full text-center">
          <div className="text-slate-500">
            <div className="text-4xl mb-2 opacity-40">🌙</div>
            <p className="text-sm">Ask me anything to start the conversation</p>
          </div>
        </div>
      ) : (
        messages.map((msg, idx) => (
          <div
            key={msg.id ?? idx}
            className={`max-w-4xl mx-auto flex flex-col gap-3 rounded-3xl border p-4 shadow-lg text-sm sm:text-base animate-fade-in ${
              msg.sender === "user"
                ? "bg-blue-950/80 text-blue-100 border-slate-800/50 items-end"
                : msg.sender === "system"
                  ? "bg-slate-800/50 text-slate-300 border-slate-700/50 items-start"
                  : "bg-slate-900/80 text-slate-100 border-l-4 border-amber-500/60 items-start"
            }`}
            style={{
              animationDuration: "0.2s",
              animationFillMode: "both",
              animationTimingFunction: "ease-out",
            }}
            role="article"
            aria-label={`Message from ${msg.sender}`}
          >
            {msg.files && msg.files.length > 0 && (
              <div className="space-y-1 text-xs text-slate-300">
                <div className="font-semibold text-slate-100">📎 Linked Files</div>
                {msg.files.map((file, index) => (
                  <div
                    key={index}
                    className="rounded-md bg-slate-800 px-2 py-1"
                  >
                    {file.name}
                  </div>
                ))}
              </div>
            )}

            {msg.screenshots && msg.screenshots.length > 0 && (
              <div className="grid gap-3 md:grid-cols-2">
                {msg.screenshots.map((shot) => (
                  <div
                    key={shot.id}
                    className="overflow-hidden rounded-2xl border border-slate-800 bg-slate-950"
                  >
                    <div className="px-3 py-2 text-xs text-slate-300 bg-slate-900">
                      {shot.label ?? `Screen ${shot.id + 1}`}
                    </div>
                    <img
                      src={shot.dataUrl}
                      alt={shot.label ?? `Screenshot ${shot.id}`}
                      className="w-full object-contain"
                      loading="lazy"
                    />
                  </div>
                ))}
              </div>
            )}

            {msg.reasoning && msg.reasoning.trim().length > 0 && (
              <div className="mb-3 rounded-xl border border-slate-700/60 bg-slate-800/40 px-4 py-3 text-sm text-slate-400 italic whitespace-pre-wrap">
                <div className="text-xs font-semibold text-slate-500 mb-1 not-italic">Thinking</div>
                {msg.reasoning}
              </div>
            )}

            <div className="prose prose-invert max-w-none break-words">
              {msg.sender === "system" ? (
                // System messages like recalled memories - rendered as plain styled text
                <p className="text-sm italic text-blue-300/80 whitespace-pre-wrap">
                  {msg.text}
                </p>
              ) : shouldRenderMarkdown(msg.text) ? (
                <ReactMarkdown
                  remarkPlugins={[remarkMath]}
                  rehypePlugins={[rehypeKatex]}
                  components={{
                    p: ({ children }) => (
                      <p className="mb-2 last:mb-0">{children}</p>
                    ),
                    h1: ({ children }) => (
                      <h1 className="text-xl font-semibold mb-2 text-amber-300">
                        {children}
                      </h1>
                    ),
                    h2: ({ children }) => (
                      <h2 className="text-lg font-semibold mb-2 text-amber-300">
                        {children}
                      </h2>
                    ),
                    h3: ({ children }) => (
                      <h3 className="text-base font-semibold mb-2 text-amber-300">
                        {children}
                      </h3>
                    ),
                    ul: ({ children }) => (
                      <ul className="list-disc list-inside mb-2 space-y-1">
                        {children}
                      </ul>
                    ),
                    ol: ({ children }) => (
                      <ol className="list-decimal list-inside mb-2 space-y-1">
                        {children}
                      </ol>
                    ),
                    li: ({ children }) => <li className="ml-4">{children}</li>,
                    code: ({ children }) => (
                      <code className="rounded bg-slate-800 px-1 py-0.5 text-sm font-mono">
                        {children}
                      </code>
                    ),
                    pre: ({ children }) => (
                      <pre className="rounded-2xl bg-slate-900 p-3 text-sm overflow-x-auto">
                        {children}
                      </pre>
                    ),
                    blockquote: ({ children }) => (
                      <blockquote className="border-l-4 border-slate-600 pl-4 italic text-slate-300 mb-2">
                        {children}
                      </blockquote>
                    ),
                    strong: ({ children }) => (
                      <strong className="font-semibold text-amber-300">
                        {children}
                      </strong>
                    ),
                    em: ({ children }) => (
                      <em className="italic text-slate-300">{children}</em>
                    ),
                  }}
                >
                  {msg.text}
                </ReactMarkdown>
              ) : (
                <pre className="whitespace-pre-wrap break-words text-sm">
                  {msg.text}
                </pre>
              )}
            </div>
          </div>
        ))
      )}
    </div>
  );
};

export default MessageList;