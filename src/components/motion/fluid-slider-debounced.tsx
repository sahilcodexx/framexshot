"use client";

import { useEffect, useRef } from "react";
import { FluidSlider, type FluidSliderProps } from "./range-slider-fluid";

export interface FluidSliderDebouncedProps
  extends Omit<FluidSliderProps, "onValueChange"> {
  /** Fires on every value change while dragging — for transient preview updates. */
  onValueChangeTransient?: (value: number) => void;
  /** Fires after the value settles (debounceMs after the last change) — pushes to history. */
  onValueCommit?: (value: number) => void;
  /** Fires when drag state changes — signals the preview generator to skip work. */
  onDragChange?: (dragging: boolean) => void;
  /** Idle window before commit. Default 150ms — keeps one drag as one undo step. */
  debounceMs?: number;
}

/**
 * FluidSlider with a transient/commit split and an isDragging flag.
 *
 * FluidSlider only exposes `onValueChange` (fires on every drag pixel). The
 * app needs three things from a slider, in order:
 *
 *   1. A *transient* update so the canvas preview tracks the cursor
 *      (e.g. `setBlurAmountTransient` — does not push to history).
 *   2. A *commit* update after the value settles so undo/redo sees one step
 *      per drag, not one per pixel.
 *   3. An `isDragging` flag so `usePreviewGenerator` can skip the expensive
 *      canvas regen while the user is mid-drag.
 *
 * This wrapper wires all three: transient + flag on every change, commit +
 * flag-clear after `debounceMs` of idle. The spring + scale animation inside
 * FluidSlider stays untouched.
 */
export function FluidSliderDebounced({
  onValueChangeTransient,
  onValueCommit,
  onDragChange,
  debounceMs = 150,
  ...rest
}: FluidSliderDebouncedProps) {
  const commitTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clear any pending commit on unmount so we never commit a value into a
  // torn-down store.
  useEffect(
    () => () => {
      if (commitTimerRef.current) clearTimeout(commitTimerRef.current);
    },
    [],
  );

  return (
    <FluidSlider
      {...rest}
      onValueChange={(v) => {
        onValueChangeTransient?.(v);
        onDragChange?.(true);
        if (commitTimerRef.current) clearTimeout(commitTimerRef.current);
        commitTimerRef.current = setTimeout(() => {
          onValueCommit?.(v);
          onDragChange?.(false);
          commitTimerRef.current = null;
        }, debounceMs);
      }}
    />
  );
}
