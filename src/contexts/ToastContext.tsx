import React, { createContext, useContext, useCallback, useRef, useEffect, useState } from "react";
import Toast from "../components/ui/Toast";

type ToastType = "info" | "success" | "warning" | "error";

interface ToastContextType {
  showToast: (message: string, options?: { type?: ToastType; duration?: number }) => void;
  hideToast: () => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toast, setToast] = useState<{ id: number; message: string; type: ToastType; duration?: number } | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showToast = useCallback((message: string, options?: { type?: ToastType; duration?: number }) => {
    const id = Date.now();
    if (timeoutRef.current) clearTimeout(timeoutRef.current);

    setToast({
      id,
      message,
      type: (options?.type as ToastType) || "info",
      duration: options?.duration,
    });

    if (options?.duration !== Infinity) {
      timeoutRef.current = setTimeout(() => {
        setToast(null);
      }, options?.duration ?? 4000);
    }
  }, []);

  const hideToast = useCallback(() => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setToast(null);
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  return (
    <ToastContext.Provider value={{ showToast, hideToast }}>
      {children}
      {toast && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
          <Toast
            message={toast.message}
            type={toast.type}
            onClose={hideToast}
          />
        </div>
      )}
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextType {
  const context = useContext(ToastContext);
  if (context === undefined) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}