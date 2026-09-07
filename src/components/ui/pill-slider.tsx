import {
  type ChangeEvent,
  useEffect,
  useRef,
  useState,
} from "react";
import {
  animate,
  motion,
  useSpring,
  useTransform,
} from "motion/react";
import { cn } from "@/lib/utils";

/**
 * PillSlider — label lives *inside* the pill (clipped to the filled portion),
 * value lives *outside* on the right. Dark-theme adaptation of the reference:
 *
 *   ┌──────────────┬─────┐
 *   │ Padding      │ 15% │  ← label inside the filled area, value outside
 *   └──────────────┴─────┘
 *
 * Interaction uses a native `<input type="range">` (opacity: 0) so we get
 * keyboard support, focus management, and touch handling for free. The
 * visible thumb + filled portion are absolutely positioned and update on
 * every value change.
 *
 * ## Why the visual is driven by local state, not the `value` prop
 *
 * If the visual position is derived from `value`, the flow on every drag
 * pixel is:
 *
 *   input.onChange → onChangeTransient → store update
 *     → parent re-renders → PillSlider re-renders with new prop
 *     → thumb position updates
 *
 * That round-trip is what makes the thumb visibly chase the cursor — every
 * store update has to walk back through React before the visual can move.
 *
 * Instead, during a drag we read directly from the native input's value via
 * a ref, mutate the thumb/fill `<div>` styles imperatively, and only commit
 * the final value to the store on pointer-up. The cursor and the visual move
 * together in the same paint frame; React reconciliation runs at its own
 * pace in the background.
 *
 * `onChangeTransient` still fires on every drag pixel (so the canvas
 * preview can update), but the component no longer *waits* for the store
 * to update before moving the thumb.
 *
 * `onChange` fires on commit (pointer up / key up) — that's what pushes
 * to history.
 */
interface PillSliderProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  /** Right-aligned display value. Falls back to the raw `value` prop. */
  displayValue?: string;
  /** Fires while the user is dragging — for visual feedback without history. */
  onChangeTransient?: (value: number) => void;
  /** Fires on commit (pointer up / key up) — pushes to history. */
  onChange?: (value: number) => void;
  /**
   * Fires when the drag state changes. Lets the caller signal the rest of
   * the app (e.g. the preview generator) to skip expensive work during the
   * drag and re-render on commit.
   */
  onDragChange?: (dragging: boolean) => void;
  className?: string;
  disabled?: boolean;
  /** ARIA value text for screen readers. */
  ariaValueText?: string;
}

export function PillSlider({
  label,
  value,
  min = 0,
  max = 100,
  step = 1,
  displayValue,
  onChangeTransient,
  onChange,
  onDragChange,
  className,
  disabled = false,
  ariaValueText,
}: PillSliderProps) {
  // ─── Refs for imperative updates during a drag ──────────────────────────
  const fillRef = useRef<HTMLDivElement | null>(null);
  const thumbRef = useRef<HTMLDivElement | null>(null);
  const valueSpanRef = useRef<HTMLSpanElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const lastDragValueRef = useRef<number>(value);

  // When non-null, the user is actively interacting with the slider and
  // the visual should follow the drag value, not the (possibly stale) prop.
  const [dragValue, setDragValue] = useState<number | null>(null);

  // Display value: local drag state wins during a drag, the prop wins
  // before/after. This is the source of truth for the rendered position.
  const displayValueNum = dragValue ?? value;
  const percent = Math.max(
    0,
    Math.min(100, ((displayValueNum - min) / (max - min)) * 100)
  );

  // Spring-driven percent. During a drag, the spring is set directly
  // (so the visual tracks the cursor 1:1). When the prop changes externally
  // — undo, redo, set-as-default, store init — the spring animates the
  // visual from its current value to the new target with rubbery physics.
  // That gives the "stretch" feel on commit without adding latency during
  // the drag itself.
  const springPercent = useSpring(percent, {
    stiffness: 320,
    damping: 28,
    mass: 0.9,
  });
  // While dragging, use the raw value (no spring tracking) so the thumb
  // never lags the cursor. When the drag ends, the spring eases into the
  // final committed value, which is the same as `percent` so it lands
  // exactly where the user let go.
  useEffect(() => {
    if (dragValue === null) springPercent.set(percent);
  }, [percent, dragValue, springPercent]);

  // Derived motion values for the fill and thumb — read from the spring
  // so they animate smoothly when the prop changes.
  const fillWidth = useTransform(springPercent, (p) => `${p}%`);
  const thumbLeft = useTransform(springPercent, (p) => `calc(${p}% - 1px)`);
  // Spring for the displayed number — rubbery count-up/down on commit.
  const springValue = useSpring(displayValueNum, {
    stiffness: 380,
    damping: 32,
    mass: 0.7,
  });
  const [shownValue, setShownValue] = useState<number>(displayValueNum);
  useEffect(() => {
    if (dragValue === null) {
      const controls = animate(springValue, displayValueNum, {
        type: "spring",
        stiffness: 380,
        damping: 32,
        mass: 0.7,
      });
      return controls.stop;
    }
  }, [displayValueNum, dragValue, springValue]);
  useEffect(() => {
    return springValue.on("change", (v) => setShownValue(v));
  }, [springValue]);

  // Imperatively write the new position to the DOM. Bypasses React
  // reconciliation for the parts that move every frame, so the cursor and
  // the visual never drift apart.
  const applyVisual = (next: number) => {
    const p = Math.max(0, Math.min(100, ((next - min) / (max - min)) * 100));
    // During a drag, set the springs directly (no easing) so the visual
    // tracks the cursor. The spring is the same one that animates on
    // commit, so when the drag ends the springs are already at the final
    // value and nothing needs to settle.
    springPercent.set(p);
    springValue.set(next);
    if (valueSpanRef.current) valueSpanRef.current.textContent = formatDisplay(next);
  };

  const formatDisplay = (v: number) => {
    if (displayValue) return displayValue;
    if (typeof v === "number" && Number.isFinite(v)) {
      // Mirror the prop's display value if it's a percentage-like number.
      // Falls back to the raw number for callers that don't pass displayValue.
      return String(Math.round(v * 100) / 100);
    }
    return String(v);
  };

  // (No imperative re-sync block needed: the `useEffect` above writes
  // the spring's value whenever `percent` changes outside a drag, and
  // the spring's motion values drive the fill + thumb styles directly.)

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = Number(e.target.value);
    if (lastDragValueRef.current !== newValue && dragValue === null) {
      // First change in this drag — signal drag start.
      onDragChange?.(true);
    }
    lastDragValueRef.current = newValue;
    setDragValue(newValue);
    applyVisual(newValue);
    onChangeTransient?.(newValue);
  };

  const handlePointerDown = () => {
    // First interaction with this slider — signal drag start.
    onDragChange?.(true);
  };

  const handleCommit = () => {
    const finalValue = lastDragValueRef.current;
    setDragValue(null);
    // Re-sync the visual to the (now committed) prop in case the store
    // round-tripped back with a clamped / normalized value.
    requestAnimationFrame(() => applyVisual(value));
    onChange?.(finalValue);
    // Drag is over — signal drag end.
    onDragChange?.(false);
  };

  return (
    <div className={cn("flex items-center gap-3 min-w-0", className)}>
      {/* Pill container — the slider track, the filled portion, the label, and the thumb all live inside.
          The motion wrapper gives the pill a spring-driven scale on focus/hover/tap, so it "stretches"
          outward when the user reaches for it (matching the reference layout where the active slider
          visibly grows while the others compress). */}
      <motion.div
        layout
        transition={{ type: "spring", stiffness: 380, damping: 30, mass: 0.85 }}
        whileHover={{ scale: 1.012 }}
        whileFocus={{ scale: 1.018 }}
        whileTap={{ scale: 0.992 }}
        className={cn(
          "relative h-9 flex-1 min-w-0 overflow-hidden rounded-full select-none",
          "bg-secondary border border-border",
          "transition-colors duration-[var(--duration-quick)]",
          "hover:border-border/80",
          "focus-within:border-accent/40",
          disabled && "opacity-50"
        )}
      >
        {/* Filled portion — width is driven by the spring `fillWidth` motion
            value. During a drag we set the spring directly (no easing) so
            the fill tracks the cursor. When the prop changes externally,
            the spring animates the fill into the new position. */}
        <motion.div
          ref={fillRef}
          className="pointer-events-none absolute inset-y-0 left-0 bg-foreground/[0.07]"
          style={{ width: fillWidth }}
        />

        {/* Label inside the pill. Sits in the filled area; clips via overflow-hidden if value is very small. */}
        <span
          className={cn(
            "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 z-10",
            "select-none whitespace-nowrap text-xs font-medium text-foreground"
          )}
        >
          {label}
        </span>

        {/* Invisible native range — captures all interaction. */}
        <input
          ref={inputRef}
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          disabled={disabled}
          onChange={handleChange}
          onPointerDown={handlePointerDown}
          onPointerUp={handleCommit}
          onKeyUp={handleCommit}
          aria-label={label}
          aria-valuetext={ariaValueText ?? displayValue ?? String(value)}
          className="absolute inset-0 h-full w-full cursor-ew-resize appearance-none bg-transparent opacity-0"
        />

        {/* Visible thumb — thin vertical pill at the current value, driven
            by the spring. Same drag/springs contract as the fill. */}
        <motion.div
          ref={thumbRef}
          className={cn(
            "pointer-events-none absolute top-1/2 -translate-y-1/2",
            "h-4 w-[2px] rounded-full bg-foreground",
            "shadow-[0_0_0_0.5px_rgba(0,0,0,0.15),0_1px_2px_rgba(0,0,0,0.1)]"
          )}
          style={{ left: thumbLeft }}
        />
      </motion.div>

      {/* Value — outside the pill, monospace for stable width. The displayed
          number is the spring-driven `shownValue` (rubbery count-up/down
          when the prop changes), with the user's formatted `displayValue`
          taking priority when provided. */}
      <span
        ref={valueSpanRef}
        className={cn(
          "shrink-0 min-w-[3ch] text-right",
          "text-xs font-mono tabular-nums text-muted-foreground"
        )}
      >
        {displayValue ?? Math.round(shownValue * 100) / 100}
      </span>
    </div>
  );
}
