import { memo } from "react";
import { Slider } from "@/components/ui/slider";
import { Eye, EyeOff, Sun } from "lucide-react";
import type { ShadowPresetId } from "@/lib/frame-presets";
import { SHADOW_PRESETS } from "@/lib/frame-presets";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface ShadowPresetsProps {
  shadowPreset: ShadowPresetId;
  opacity: number;
  showMockup: boolean;
  onPresetChange: (preset: ShadowPresetId) => void;
  onOpacityChangeTransient?: (value: number) => void;
  onOpacityChange: (value: number) => void;
  onToggleMockup: () => void;
  /** When true, show the advanced blur/offset sliders below */
  showAdvanced?: boolean;
  children?: React.ReactNode;
}

/** Human-friendly names and descriptions for shadow presets */
const SHADOW_META: Record<ShadowPresetId, { label: string; description: string }> = {
  none:     { label: "None",    description: "No shadow — flat and clean." },
  spread:   { label: "Lifted",  description: "A wide, soft shadow — looks like it's floating above the surface." },
  hug:      { label: "Close",   description: "A tight shadow right under the edges — subtle and precise." },
  adaptive: { label: "Vivid",   description: "A coloured shadow that adapts to your background — dramatic and eye-catching." },
};

function ShadowSwatch({ id, active }: { id: ShadowPresetId; active: boolean }) {
  const ring = active
    ? "border-white/70 ring-2 ring-white/30 scale-[0.97]"
    : "border-[#2a2a2a] hover:border-[#555] hover:scale-[0.97]";

  const base = `relative w-full aspect-square rounded-xl border bg-[#2a2a2a] overflow-hidden transition-all ${ring}`;

  switch (id) {
    case "none":
      return (
        <div className={base}>
          {/* flat card — no shadow */}
          <div className="absolute inset-[22%] rounded-md bg-white" />
        </div>
      );
    case "spread":
      return (
        <div className={base}>
          {/* wide diffuse shadow underneath */}
          <div className="absolute inset-[22%] rounded-md bg-white shadow-[0_10px_28px_4px_rgba(0,0,0,0.70)]" />
        </div>
      );
    case "hug":
      return (
        <div className={base}>
          {/* tight shadow hugging all edges */}
          <div className="absolute inset-[22%] rounded-md bg-white shadow-[0_2px_6px_1px_rgba(0,0,0,0.50)]" />
        </div>
      );
    case "adaptive":
      return (
        <div className={`${base} bg-gradient-to-br from-[#ff7a5c] to-[#c44]`}>
          {/* coloured shadow matching background */}
          <div className="absolute inset-[22%] rounded-md bg-white/90 shadow-[6px_14px_30px_rgba(140,30,20,0.65)]" />
        </div>
      );
    default:
      return <div className={base} />;
  }
}

export const ShadowPresets = memo(function ShadowPresets({
  shadowPreset,
  opacity,
  showMockup,
  onPresetChange,
  onOpacityChangeTransient,
  onOpacityChange,
  onToggleMockup,
  children,
}: ShadowPresetsProps) {
  return (
    <div className="space-y-5">
      {/* Preset grid */}
      <TooltipProvider delayDuration={300}>
        <div className="grid grid-cols-4 gap-2">
          {SHADOW_PRESETS.map((preset) => {
            const active = shadowPreset === preset.id;
            const meta = SHADOW_META[preset.id];
            return (
              <Tooltip key={preset.id}>
                <TooltipTrigger asChild>
                  <button
                    type="button"
                    onClick={() => onPresetChange(preset.id)}
                    aria-pressed={active}
                    className="flex flex-col items-center gap-1.5 group"
                  >
                    <ShadowSwatch id={preset.id} active={active} />
                    <span
                      className={`text-[10px] font-medium truncate w-full text-center transition-colors ${
                        active ? "text-white" : "text-muted-foreground group-hover:text-white/80"
                      }`}
                    >
                      {meta.label}
                    </span>
                  </button>
                </TooltipTrigger>
                <TooltipContent side="bottom" className="max-w-[180px] text-xs text-center leading-relaxed">
                  <p className="font-semibold mb-0.5">{meta.label}</p>
                  <p className="text-muted-foreground">{meta.description}</p>
                </TooltipContent>
              </Tooltip>
            );
          })}
        </div>
      </TooltipProvider>

      {/* Opacity */}
      {shadowPreset !== "none" && (
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-xs text-muted-foreground font-medium">Strength</label>
            <span className="text-xs text-muted-foreground font-mono tabular-nums">{opacity}%</span>
          </div>
          <Slider
            value={[opacity]}
            onValueChange={(value) => onOpacityChangeTransient?.(value[0])}
            onValueCommit={(value) => onOpacityChange(value[0])}
            min={0}
            max={100}
            step={1}
            className="w-full"
          />
        </div>
      )}

      {/* Advanced sliders (blur / offset) passed as children */}
      {children}

      {/* Visibility — Hide Mockup */}
      <div className="space-y-2 pt-1">
        <h4 className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
          Window Bar
        </h4>
        <button
          type="button"
          onClick={onToggleMockup}
          className={`
            w-full flex items-center justify-center gap-2 h-9 rounded-full border text-xs font-medium transition-colors
            ${showMockup
              ? "bg-[#1a1a1a] border-[#2a2a2a] text-muted-foreground hover:text-white hover:border-[#444]"
              : "bg-white text-black border-white"}
          `}
        >
          {showMockup ? (
            <>
              <EyeOff className="size-3.5" aria-hidden="true" />
              Hide window bar
            </>
          ) : (
            <>
              <Eye className="size-3.5" aria-hidden="true" />
              Show window bar
            </>
          )}
        </button>
      </div>

      {/* Details footer (informational, matches reference) */}
      <div className="space-y-2 pt-1 border-t border-[#1a1a1a]">
        <h4 className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
          Details
        </h4>
        <div className="grid grid-cols-2 gap-2 text-[11px]">
          <div className="rounded-lg bg-[#141414] border border-[#1a1a1a] px-2.5 py-2">
            <div className="text-muted-foreground">Device</div>
            <div className="text-foreground font-medium mt-0.5">Screen pixels</div>
          </div>
          <div className="rounded-lg bg-[#141414] border border-[#1a1a1a] px-2.5 py-2">
            <div className="text-muted-foreground flex items-center gap-1">
              <Sun className="size-3" aria-hidden="true" />
              Screenshot
            </div>
            <div className="text-foreground font-medium mt-0.5">Adapts to media</div>
          </div>
        </div>
      </div>
    </div>
  );
});
