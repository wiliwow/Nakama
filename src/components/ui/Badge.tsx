import React from "react";

interface BadgeProps {
  count?: number;
  children?: React.ReactNode;
  variant?: "default" | "success" | "warning" | "error";
  size?: "sm" | "md";
}

const Badge: React.FC<BadgeProps> = ({ count, children, variant = "default", size = "sm" }) => {
  const variants = {
    default: "bg-slate-700 text-slate-200",
    success: "bg-emerald-600 text-white",
    warning: "bg-amber-600 text-white",
    error: "bg-red-600 text-white"
  };

  const sizes = {
    sm: "min-w-[18px] h-[18px] text-xs",
    md: "min-w-[22px] h-[22px] text-sm"
  };

  // If there are children, render as badge overlay
  if (children) {
    return (
      <div className="relative inline-flex items-center">
        {children}
        {count !== undefined && count > 0 && (
          <span className={`absolute -top-1 -right-1 rounded-full font-bold flex items-center justify-center ${variants[variant]} ${sizes[size]}`}>
            {count > 99 ? "99+" : count}
          </span>
        )}
      </div>
    );
  }

  // Standalone badge
  return (
    <span className={`inline-flex items-center justify-center rounded-full font-bold px-2 ${variants[variant]} ${sizes[size]}`}>
      {count !== undefined ? (count > 99 ? "99+" : count) : ""}
    </span>
  );
};

export default Badge;
