import { Minus, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

export function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-9 shrink-0 bg-canvas border-b border-border select-none"
    >
      <div className="flex items-center gap-2 px-4">
        <span className="text-xs font-medium tracking-[-0.01em] text-muted-foreground">FrameXShot</span>
      </div>
      <div className="flex items-center">
        <button
          onClick={() => appWindow.minimize()}
          className="flex items-center justify-center size-9 text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
          aria-label="Minimize"
        >
          <Minus className="size-3.5" aria-hidden="true" />
        </button>
        <button
          onClick={() => appWindow.close()}
          className="flex items-center justify-center size-9 text-muted-foreground hover:text-foreground hover:bg-destructive/10 transition-colors"
          aria-label="Close"
        >
          <X className="size-3.5" aria-hidden="true" />
        </button>
      </div>
    </div>
  );
}
