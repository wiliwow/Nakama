import React from "react";
import Tabs from "./ui/Tabs";
import Card from "./ui/Card";
import Spinner from "./ui/Spinner";
import ScreenVisionPanel from "./ScreenVisionPanel";
import AutomationControlPanel from "./AutomationControlPanel";

interface SidePanelProps {
  screenVisionEnabled: boolean;
  screenCaptures: any[];
  screenCaptureLoading: "primary" | "all" | null;
  onScreenVisionToggle: () => void;
  onCapturePrimary: () => void;
  onCaptureAll: () => void;
}

const SidePanel: React.FC<SidePanelProps> = ({
  screenVisionEnabled,
  screenCaptures,
  screenCaptureLoading,
  onScreenVisionToggle,
  onCapturePrimary,
  onCaptureAll
}) => {
  const [activeTab, setActiveTab] = React.useState("vision");

  return (
    <aside className="hidden md:flex md:w-[340px] lg:w-[380px] flex-col h-full bg-slate-900/20 border-l border-slate-800/40">
      <div className="flex-1 min-h-0 flex flex-col">
        {/* Tabs */}
        <div className="px-4 pt-4">
          <Tabs
            tabs={[
              { id: "vision", label: "Screen Vision", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg> },
              { id: "automation", label: "Automation", icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg> }
            ]}
            activeTab={activeTab}
            onChange={setActiveTab}
          />
        </div>

        {/* Panel Content */}
        <div className="flex-1 min-h-0 overflow-y-auto px-4 pb-4 space-y-4">
          {activeTab === "vision" && (
            <Card title="Screen Vision" collapsible defaultCollapsed={false}>
              <ScreenVisionPanel
                enabled={screenVisionEnabled}
                captures={screenCaptures}
                onToggleEnabled={onScreenVisionToggle}
                onCapturePrimary={onCapturePrimary}
                onCaptureAll={onCaptureAll}
                loading={screenCaptureLoading}
              />
            </Card>
          )}

          {activeTab === "automation" && (
            <Card title="Automation Controls" collapsible defaultCollapsed={false}>
              <AutomationControlPanel />
            </Card>
          )}
        </div>
      </div>

      {/* Optional status indicator */}
      {screenCaptureLoading && (
        <div className="px-4 py-2 border-t border-slate-800/30">
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <Spinner size="sm" />
            Processing...
          </div>
        </div>
      )}
    </aside>
  );
};

export default SidePanel;
