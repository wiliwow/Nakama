import React from "react";

interface CardProps {
  children: React.ReactNode;
  className?: string;
  title?: string;
  actions?: React.ReactNode;
  collapsible?: boolean;
  defaultCollapsed?: boolean;
}

const Card: React.FC<CardProps> = ({ children, className = "", title, actions, collapsible = false, defaultCollapsed = false }) => {
  const [collapsed, setCollapsed] = React.useState(defaultCollapsed);

  return (
    <div className={`bg-[#1b2140] border border-blue-900 rounded-2xl overflow-hidden ${className}`}>
      {title && (
        <div className="flex items-center justify-between px-4 py-3 border-b border-blue-900/50">
          <h3 className="text-sm font-semibold text-yellow-200">{title}</h3>
          {actions && <div className="flex items-center gap-2">{actions}</div>}
          {collapsible && (
            <button
              onClick={() => setCollapsed(!collapsed)}
              className="ml-2 text-blue-400 hover:text-blue-300 transition-colors"
              aria-label={collapsed ? "Expand" : "Collapse"}
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={collapsed ? "M19 9l-7 7-7-7" : "M9 5l7 7-7 7"} />
              </svg>
            </button>
          )}
        </div>
      )}
      {(!collapsible || !collapsed) && <div className="p-4">{children}</div>}
    </div>
  );
};

export default Card;
