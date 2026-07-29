import { memo } from "react";
import { Monitor, SquareTerminal, Image as ImageIcon } from "lucide-react";
import { WindowFrameType } from "@/stores/editorStore";

interface MockupSelectorProps {
  windowFrame: WindowFrameType;
  onChange: (frame: WindowFrameType) => void;
}

const mockups: Array<{ id: WindowFrameType; label: string; icon: React.ReactNode }> = [
  { id: "none", label: "None", icon: <ImageIcon className="size-4 mb-1 text-muted-foreground" /> },
  { id: "macos", label: "macOS", icon: <SquareTerminal className="size-4 mb-1 text-muted-foreground" /> },
  { id: "windows", label: "Windows", icon: <Monitor className="size-4 mb-1 text-muted-foreground" /> }
];

export const MockupSelector = memo(function MockupSelector({ windowFrame, onChange }: MockupSelectorProps) {
  return (
    <div className="grid grid-cols-3 gap-2">
      {mockups.map((mockup) => (
        <button
          key={mockup.id}
          onClick={() => onChange(mockup.id)}
          className={`
            flex flex-col items-center justify-center py-2 px-1 rounded-lg border text-xs transition-colors
            ${windowFrame === mockup.id 
              ? "bg-[#1a1a1a] border-[#333] text-white" 
              : "bg-[#0a0a0a] border-transparent text-muted-foreground hover:bg-[#141414] hover:text-white"}
          `}
        >
          {mockup.icon}
          {mockup.label}
        </button>
      ))}
    </div>
  );
});
