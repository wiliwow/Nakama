import React from "react";

interface ToastProps {
  message: string;
  type?: "info" | "success" | "warning" | "error";
  onClose?: () => void;
}

const Toast: React.FC<ToastProps> = ({ message, type = "info", onClose }) => {
  const types = {
    info: "bg-blue-600",
    success: "bg-emerald-600",
    warning: "bg-amber-600",
    error: "bg-red-600",
  };

  return (
    <div
      className={`${types[type]} text-white px-4 py-3 rounded-xl shadow-lg flex items-center justify-between gap-3`}
      role="alert"
    >
      <span className="text-sm">{message}</span>
      {onClose && (
        <button
          onClick={onClose}
          className="text-white/80 hover:text-white"
          aria-label="Close toast"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      )}
    </div>
  );
};

export default Toast;