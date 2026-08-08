import { useRef, useEffect, useCallback, useState, useMemo } from "react";
import { EditorSettings } from "@/stores/editorStore";
import { drawAnnotationOnCanvas } from "@/lib/annotation-utils";
import { Annotation } from "@/types/annotations";
import {
  getFrameStyle,
  getLayoutTransform,
  applyFrameStyle,
  applyLayoutTransform,
  layoutPaddingFactor,
  isLayoutTransformed,
} from "@/lib/frame-presets";

import { resolveBackgroundPath, getAssetPath } from "@/lib/asset-registry";
import { gradientOptions } from "@/components/editor/BackgroundSelector";

// Image cache with LRU-like cleanup (max 20 images)
const MAX_CACHE_SIZE = 20;
const imageCache = new Map<string, HTMLImageElement>();
const cacheOrder: string[] = [];

function addToCache(src: string, img: HTMLImageElement) {
  if (imageCache.size >= MAX_CACHE_SIZE) {
    const oldest = cacheOrder.shift();
    if (oldest) {
      imageCache.delete(oldest);
    }
  }
  imageCache.set(src, img);
  cacheOrder.push(src);
}

/**
 * Load an image from a URL, using cache if available
 */
export async function loadImage(src: string): Promise<HTMLImageElement> {
  if (imageCache.has(src)) {
    return imageCache.get(src)!;
  }

  let finalSrc = src;

  // Convert absolute file system paths or asset protocol URLs to base64 data URIs
  if (!src.startsWith("data:") && !src.startsWith("http:") && !src.startsWith("https:") && (src.startsWith("/") || src.startsWith("asset:") || src.includes("tmp"))) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      let filePath = src;
      if (src.startsWith("asset:")) {
        filePath = decodeURIComponent(src.replace(/^asset:\/\/[^\/]+\//, "/"));
      }
      finalSrc = await invoke<string>("read_file_as_base64", { path: filePath });
    } catch (e) {
      console.warn("Base64 image conversion failed, falling back to original src:", e);
    }
  }

  return new Promise((resolve, reject) => {
    const img = new Image();

    if (finalSrc.startsWith("http") || finalSrc.startsWith("asset:")) {
      img.crossOrigin = "anonymous";
    }

    img.onload = () => {
      addToCache(src, img);
      resolve(img);
    };

    img.onerror = (event) => {
      const error = new Error(`Failed to load image: ${src}`);
      console.error("Image load error:", { src, event });
      reject(error);
    };

    img.src = finalSrc;
  });
}

/**
 * Get the background image source based on settings
 */
export function getBackgroundImageSrc(settings: EditorSettings): string | null {
  if (settings.backgroundType === "image" && settings.selectedImageSrc) {
    return resolveBackgroundPath(settings.selectedImageSrc);
  }
  if (settings.backgroundType === "gradient") {
    if (settings.gradientSrc) {
      const resolved = resolveBackgroundPath(settings.gradientSrc);
      if (resolved) return resolved;
    }
    if (settings.gradientId) {
      const assetId = settings.gradientId.replace("mesh-", "gradient-");
      const path = getAssetPath(assetId);
      if (path) return path;
    }
    const gradOpt = gradientOptions.find((g) => g.id === settings.gradientId);
    if (gradOpt) return gradOpt.src;
  }
  return null;
}

/**
 * Draw an image onto canvas using cover fitting (maintains aspect ratio, crops overflow)
 */
function drawImageCover(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  targetWidth: number,
  targetHeight: number
) {
  const imgRatio = img.width / img.height;
  const targetRatio = targetWidth / targetHeight;

  let sourceX = 0;
  let sourceY = 0;
  let sourceWidth = img.width;
  let sourceHeight = img.height;

  if (imgRatio > targetRatio) {
    sourceWidth = img.height * targetRatio;
    sourceX = (img.width - sourceWidth) / 2;
  } else {
    sourceHeight = img.width / targetRatio;
    sourceY = (img.height - sourceHeight) / 2;
  }

  ctx.drawImage(
    img,
    sourceX,
    sourceY,
    sourceWidth,
    sourceHeight,
    0,
    0,
    targetWidth,
    targetHeight
  );
}

/**
 * Draw background on a canvas context
 */
function drawBackground(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  settings: EditorSettings,
  bgImage: HTMLImageElement | null
) {
  switch (settings.backgroundType) {
    case "transparent": {
      break;
    }
    case "white":
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, width, height);
      break;
    case "black":
      ctx.fillStyle = "#000000";
      ctx.fillRect(0, 0, width, height);
      break;
    case "gray":
      ctx.fillStyle = "#f5f5f5";
      ctx.fillRect(0, 0, width, height);
      break;
    case "gradient":
      if (bgImage) {
        drawImageCover(ctx, bgImage, width, height);
      } else {
        const gradient = ctx.createLinearGradient(0, 0, width, height);
        const colors = settings.gradientColors || ["#667eea", "#764ba2"];
        gradient.addColorStop(0, colors[0]);
        gradient.addColorStop(1, colors[1]);
        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, width, height);
      }
      break;
    case "custom":
      ctx.fillStyle = settings.customColor;
      ctx.fillRect(0, 0, width, height);
      break;
    case "image":
      if (bgImage) {
        drawImageCover(ctx, bgImage, width, height);
      } else {
        ctx.fillStyle = "#ffffff";
        ctx.fillRect(0, 0, width, height);
      }
      break;
  }
}


/**
 * Full editor composite: background + effects + framed screenshot + layout + shadow.
 * Shared by live preview and high-quality export so they stay in sync.
 */
export function renderFullCanvas(
  screenshotImage: HTMLImageElement,
  settings: EditorSettings,
  padding: { top: number; bottom: number; left: number; right: number },
  bgImage: HTMLImageElement | null,
  options: { renderEffects: boolean } = { renderEffects: true }
): HTMLCanvasElement {
  const { top: paddingTop, bottom: paddingBottom, left: paddingLeft, right: paddingRight } = padding;
  const layoutId = settings.layoutPreset || "flat";
  const extraFactor = layoutPaddingFactor(layoutId);
  const styleId = settings.frameStyle || "default";
  const styleDef = getFrameStyle(styleId);
  // Use user-overridden padding if set (framePadding >= 0), else fall back to style default
  const framePad = (settings.framePadding !== undefined && settings.framePadding >= 0)
    ? settings.framePadding
    : styleDef.padding;

  const scale = settings.imageScale ?? 1.0;
  const scaledWidth = Math.round(screenshotImage.width * scale);
  const scaledHeight = Math.round(screenshotImage.height * scale);

  const extraX = Math.round(scaledWidth * extraFactor);
  const extraY = Math.round(scaledHeight * extraFactor);
  const bgWidth =
    scaledWidth + paddingLeft + paddingRight + extraX * 2 + framePad * 2;
  const bgHeight =
    scaledHeight + paddingTop + paddingBottom + extraY * 2 + framePad * 2;
  const contentPadL = paddingLeft + extraX + framePad;
  const contentPadT = paddingTop + extraY + framePad;

  const canvas = document.createElement("canvas");
  canvas.width = bgWidth;
  canvas.height = bgHeight;
  const ctx = canvas.getContext("2d", { alpha: true });
  if (!ctx) throw new Error("Failed to get canvas context");

  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";

  const totalPadding = paddingTop + paddingBottom + paddingLeft + paddingRight;
  if (totalPadding === 0) {
    ctx.beginPath();
    ctx.roundRect(0, 0, scaledWidth, scaledHeight, settings.borderRadius);
    ctx.closePath();
    ctx.clip();
    ctx.drawImage(screenshotImage, 0, 0, scaledWidth, scaledHeight);
    return canvas;
  }

  // Background plate
  const tempCanvas = document.createElement("canvas");
  tempCanvas.width = bgWidth;
  tempCanvas.height = bgHeight;
  const tempCtx = tempCanvas.getContext("2d")!;
  drawBackground(tempCtx, bgWidth, bgHeight, settings, bgImage);

  if (options.renderEffects && settings.blurAmount > 0) {
    applyFastBoxBlur(tempCanvas, settings.blurAmount);
  }
  if (options.renderEffects && settings.noiseAmount > 0) {
    applyNoise(tempCanvas, settings.noiseAmount);
  }

  ctx.drawImage(tempCanvas, 0, 0);

  const framed = buildFramedScreenshot(screenshotImage, settings);
  if (!framed) throw new Error("Failed to build framed screenshot");

  const contentW = framed.width;
  const contentH = framed.height;

  // Independent position offset (pan) on the background canvas
  const offsetX = settings.imageOffsetX ?? 0;
  const offsetY = settings.imageOffsetY ?? 0;
  const drawX = contentPadL + offsetX;
  const drawY = contentPadT + offsetY;

  const layout = getLayoutTransform(layoutId);
  const needsTransform = isLayoutTransformed(layoutId);

  ctx.save();
  ctx.shadowColor = `rgba(0, 0, 0, ${settings.shadow.opacity / 100})`;
  ctx.shadowBlur = settings.shadow.blur;
  ctx.shadowOffsetX = settings.shadow.offsetX;
  ctx.shadowOffsetY = settings.shadow.offsetY;

  if (needsTransform) {
    const cx = drawX + contentW / 2;
    const cy = drawY + contentH / 2;
    ctx.translate(cx, cy);
    applyLayoutTransform(ctx, layout);
    ctx.drawImage(framed, -contentW / 2, -contentH / 2);
  } else {
    ctx.drawImage(framed, drawX, drawY);
  }

  ctx.shadowColor = "transparent";
  ctx.shadowBlur = 0;
  ctx.shadowOffsetX = 0;
  ctx.shadowOffsetY = 0;
  ctx.restore();

  return canvas;
}

/**
 * Build the screenshot layer with optional mockup chrome + frame style chrome.
 * Returns a canvas ready to be drawn (with shadow / layout transform applied by caller).
 */
function buildFramedScreenshot(
  screenshotImage: HTMLImageElement,
  settings: EditorSettings
): HTMLCanvasElement | null {
  const showMockup = settings.showMockup !== false;
  const frameType =
    showMockup && settings.windowFrame && settings.windowFrame !== "none"
      ? settings.windowFrame
      : "none";
  const headerHeight = frameType === "none" ? 0 : 36;
  const borderRadius = settings.borderRadius ?? 12;

  const scale = settings.imageScale ?? 1.0;
  const scaledW = Math.round(screenshotImage.width * scale);
  const scaledH = Math.round(screenshotImage.height * scale);

  const imageCanvas = document.createElement("canvas");
  imageCanvas.width = scaledW;
  imageCanvas.height = scaledH + headerHeight;
  const imageCtx = imageCanvas.getContext("2d");
  if (!imageCtx) return null;

  imageCtx.imageSmoothingEnabled = true;
  imageCtx.imageSmoothingQuality = "high";

  imageCtx.beginPath();
  imageCtx.roundRect(0, 0, imageCanvas.width, imageCanvas.height, borderRadius);
  imageCtx.closePath();
  imageCtx.clip();

  if (frameType === "macos") {
    imageCtx.fillStyle = "#1e1e1e";
    imageCtx.fillRect(0, 0, imageCanvas.width, headerHeight);

    const circleY = headerHeight / 2;
    const radius = 6;

    imageCtx.beginPath();
    imageCtx.arc(18, circleY, radius, 0, Math.PI * 2);
    imageCtx.fillStyle = "#ff5f56";
    imageCtx.fill();

    imageCtx.beginPath();
    imageCtx.arc(36, circleY, radius, 0, Math.PI * 2);
    imageCtx.fillStyle = "#ffbd2e";
    imageCtx.fill();

    imageCtx.beginPath();
    imageCtx.arc(54, circleY, radius, 0, Math.PI * 2);
    imageCtx.fillStyle = "#27c93f";
    imageCtx.fill();
  } else if (frameType === "windows") {
    imageCtx.fillStyle = "#202020";
    imageCtx.fillRect(0, 0, imageCanvas.width, headerHeight);

    imageCtx.strokeStyle = "#cccccc";
    imageCtx.lineWidth = 1.5;
    const rightX = imageCanvas.width - 20;

    imageCtx.beginPath();
    imageCtx.moveTo(rightX - 6, headerHeight / 2 - 4);
    imageCtx.lineTo(rightX + 2, headerHeight / 2 + 4);
    imageCtx.moveTo(rightX + 2, headerHeight / 2 - 4);
    imageCtx.lineTo(rightX - 6, headerHeight / 2 + 4);
    imageCtx.stroke();
  }

  // Draw screenshot image scaled to scaledW x scaledH — no cropping!
  imageCtx.drawImage(screenshotImage, 0, headerHeight, scaledW, scaledH);

  // Apply glass / inset / outline / border frame chrome
  const frameStyle = getFrameStyle(settings.frameStyle || "default");
  const framePaddingOverride = (settings.framePadding !== undefined && settings.framePadding >= 0)
    ? settings.framePadding
    : undefined;
  const opacityFactor = (settings.frameOpacity !== undefined)
    ? settings.frameOpacity / 100
    : 1;
  return applyFrameStyle(imageCanvas, frameStyle, borderRadius, {
    paddingOverride: framePaddingOverride,
    opacityFactor,
  });
}

/**
 * Fast box blur approximation — O(n) sliding window, multi-pass
 */
function applyFastBoxBlur(canvas: HTMLCanvasElement, radius: number) {
  if (radius <= 0) return;

  const passes = Math.min(Math.ceil(radius / 15) + 1, 3);
  const boxRadius = Math.floor(radius / passes);
  if (boxRadius <= 0) return;

  const ctx = canvas.getContext("2d")!;
  const width = canvas.width;
  const height = canvas.height;
  const kernelSize = boxRadius * 2 + 1;

  for (let pass = 0; pass < passes; pass++) {
    const imageData = ctx.getImageData(0, 0, width, height);
    const data = imageData.data;
    const tempData = new Uint8ClampedArray(data);

    for (let y = 0; y < height; y++) {
      let rSum = 0, gSum = 0, bSum = 0, aSum = 0;
      for (let x = 0; x < width; x++) {
        if (x === 0) {
          for (let kx = -boxRadius; kx <= boxRadius; kx++) {
            const px = Math.max(0, Math.min(width - 1, kx));
            const idx = (y * width + px) * 4;
            rSum += data[idx]; gSum += data[idx + 1]; bSum += data[idx + 2]; aSum += data[idx + 3];
          }
        } else {
          const removeX = Math.max(0, Math.min(width - 1, x - boxRadius - 1));
          const addX = Math.max(0, Math.min(width - 1, x + boxRadius));
          const removeIdx = (y * width + removeX) * 4;
          const addIdx = (y * width + addX) * 4;
          rSum = rSum - data[removeIdx] + data[addIdx];
          gSum = gSum - data[removeIdx + 1] + data[addIdx + 1];
          bSum = bSum - data[removeIdx + 2] + data[addIdx + 2];
          aSum = aSum - data[removeIdx + 3] + data[addIdx + 3];
        }
        const idx = (y * width + x) * 4;
        tempData[idx] = rSum / kernelSize | 0;
        tempData[idx + 1] = gSum / kernelSize | 0;
        tempData[idx + 2] = bSum / kernelSize | 0;
        tempData[idx + 3] = aSum / kernelSize | 0;
      }
    }

    const finalData = new Uint8ClampedArray(tempData);
    for (let x = 0; x < width; x++) {
      let rSum = 0, gSum = 0, bSum = 0, aSum = 0;
      for (let y = 0; y < height; y++) {
        if (y === 0) {
          for (let ky = -boxRadius; ky <= boxRadius; ky++) {
            const py = Math.max(0, Math.min(height - 1, ky));
            const idx = (py * width + x) * 4;
            rSum += tempData[idx]; gSum += tempData[idx + 1]; bSum += tempData[idx + 2]; aSum += tempData[idx + 3];
          }
        } else {
          const removeY = Math.max(0, Math.min(height - 1, y - boxRadius - 1));
          const addY = Math.max(0, Math.min(height - 1, y + boxRadius));
          const removeIdx = (removeY * width + x) * 4;
          const addIdx = (addY * width + x) * 4;
          rSum = rSum - tempData[removeIdx] + tempData[addIdx];
          gSum = gSum - tempData[removeIdx + 1] + tempData[addIdx + 1];
          bSum = bSum - tempData[removeIdx + 2] + tempData[addIdx + 2];
          aSum = aSum - tempData[removeIdx + 3] + tempData[addIdx + 3];
        }
        const idx = (y * width + x) * 4;
        finalData[idx] = rSum / kernelSize | 0;
        finalData[idx + 1] = gSum / kernelSize | 0;
        finalData[idx + 2] = bSum / kernelSize | 0;
        finalData[idx + 3] = aSum / kernelSize | 0;
      }
    }
    ctx.putImageData(new ImageData(finalData, width, height), 0, 0);
  }
}

/**
 * Apply noise effect to a canvas in place
 */
function applyNoise(canvas: HTMLCanvasElement, noiseAmount: number) {
  if (noiseAmount <= 0) return;

  const ctx = canvas.getContext("2d")!;
  const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
  const data = imageData.data;
  const noiseIntensity = noiseAmount * 2.55;
  const len = data.length;

  for (let i = 0; i < len; i += 4) {
    const noise = (Math.random() - 0.5) * noiseIntensity;
    data[i] = Math.max(0, Math.min(255, data[i] + noise));
    data[i + 1] = Math.max(0, Math.min(255, data[i + 1] + noise));
    data[i + 2] = Math.max(0, Math.min(255, data[i + 2] + noise));
  }
  ctx.putImageData(imageData, 0, 0);
}

export interface PreviewGeneratorOptions {
  screenshotImage: HTMLImageElement | null;
  settings: EditorSettings;
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
  paddingTop?: number;
  paddingBottom?: number;
  paddingLeft?: number;
  paddingRight?: number;
  imagePath?: string;
}

export interface PreviewGeneratorResult {
  previewUrl: string | null;
  isGenerating: boolean;
  error: string | null;
  renderHighQualityCanvas: (annotations: Annotation[], imagePath?: string) => Promise<HTMLCanvasElement | null>;
}

const PREVIEW_DEBOUNCE_MS = 16;
const EFFECTS_IDLE_MS = 200;

/**
 * Hook for generating preview images based on editor settings
 * Optimized: blur/noise only render when slider settles (idle >200ms)
 */
export function usePreviewGenerator({
  screenshotImage,
  settings,
  canvasRef,
  paddingTop = 100,
  paddingBottom = 100,
  paddingLeft = 100,
  paddingRight = 100,
  imagePath: _imagePath,
}: PreviewGeneratorOptions): PreviewGeneratorResult {
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const previewUrlRef = useRef<string | null>(null);
  const renderIdRef = useRef(0);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const effectsTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingSettingsRef = useRef<EditorSettings | null>(null);
  const renderEffectsRef = useRef(false);

  // Memoize background-related settings for comparison
  const bgSettingsKey = useMemo(() => {
    return JSON.stringify({
      backgroundType: settings.backgroundType,
      selectedImageSrc: settings.selectedImageSrc,
      gradientId: settings.gradientId,
      gradientSrc: settings.gradientSrc,
      customColor: settings.customColor,
    });
  }, [
    settings.backgroundType,
    settings.selectedImageSrc,
    settings.gradientId,
    settings.gradientSrc,
    settings.customColor,
  ]);

  // Core render function
  const generatePreview = useCallback(async (settingsToRender: EditorSettings) => {
    if (!screenshotImage || !canvasRef.current) return;

    const currentRenderId = ++renderIdRef.current;
    const canvas = canvasRef.current;
    const shouldRenderEffects = renderEffectsRef.current;

    setIsGenerating(true);
    setError(null);

    try {
      const bgSrc = getBackgroundImageSrc(settingsToRender);
      let bgImage: HTMLImageElement | null = null;
      if (bgSrc) {
        bgImage = await loadImage(bgSrc);
      }

      if (currentRenderId !== renderIdRef.current) return;

      const rendered = renderFullCanvas(
        screenshotImage,
        settingsToRender,
        { top: paddingTop, bottom: paddingBottom, left: paddingLeft, right: paddingRight },
        bgImage,
        { renderEffects: shouldRenderEffects }
      );

      if (currentRenderId !== renderIdRef.current) return;

      // Downscale preview canvas for instant live preview (max 1400px width/height)
      // Keeps slider dragging and zooming ultra-smooth at 60fps!
      const MAX_PREVIEW_DIM = 1400;
      const previewScale = Math.min(1.0, MAX_PREVIEW_DIM / Math.max(rendered.width, rendered.height));

      canvas.width = Math.round(rendered.width * previewScale);
      canvas.height = Math.round(rendered.height * previewScale);
      const ctx = canvas.getContext("2d", { alpha: true });
      if (!ctx) {
        setError("Failed to get canvas context");
        return;
      }
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(rendered, 0, 0, canvas.width, canvas.height);

      if (previewUrlRef.current) {
        URL.revokeObjectURL(previewUrlRef.current);
        previewUrlRef.current = null;
      }
      const url = canvas.toDataURL("image/jpeg", 0.80);
      previewUrlRef.current = url;
      setPreviewUrl(url);
      setIsGenerating(false);
    } catch (err) {
      if (currentRenderId === renderIdRef.current) {
        const message = err instanceof Error ? err.message : String(err);
        setError(`Preview generation failed: ${message}`);
        setIsGenerating(false);
        console.error("Preview generation failed:", err);
      }
    }
  }, [screenshotImage, canvasRef, paddingTop, paddingBottom, paddingLeft, paddingRight]);

  // Debounced preview generation + idle detection for effects
  useEffect(() => {
    if (!screenshotImage || !canvasRef.current) return;

    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    pendingSettingsRef.current = settings;

    // Reset effects flag on every settings change (during drag)
    renderEffectsRef.current = false;

    // After 200ms of no changes, enable effects and re-render
    if (effectsTimerRef.current) {
      clearTimeout(effectsTimerRef.current);
    }
    effectsTimerRef.current = setTimeout(() => {
      renderEffectsRef.current = true;
      if (pendingSettingsRef.current) {
        generatePreview(pendingSettingsRef.current);
      }
    }, EFFECTS_IDLE_MS);

    debounceTimerRef.current = setTimeout(() => {
      if (pendingSettingsRef.current) {
        generatePreview(pendingSettingsRef.current);
      }
    }, PREVIEW_DEBOUNCE_MS);

    return () => {
      if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);
      if (effectsTimerRef.current) clearTimeout(effectsTimerRef.current);
    };
  }, [
    screenshotImage,
    bgSettingsKey,
    settings.blurAmount,
    settings.noiseAmount,
    settings.borderRadius,
    settings.shadow.blur,
    settings.shadow.offsetX,
    settings.shadow.offsetY,
    settings.shadow.opacity,
    settings.windowFrame,
    settings.frameStyle,
    settings.layoutPreset,
    settings.borderPreset,
    settings.shadowPreset,
    settings.showMockup,
    settings.framePadding,
    settings.frameOpacity,
    settings.imageScale,
    settings.imageOffsetX,
    settings.imageOffsetY,
    paddingTop,
    paddingBottom,
    paddingLeft,
    paddingRight,
    canvasRef,
    generatePreview,
  ]);

  // Cleanup preview URL on unmount
  useEffect(() => {
    return () => {
      if (previewUrlRef.current) {
        URL.revokeObjectURL(previewUrlRef.current);
      }
    };
  }, []);

  // High quality canvas render for save/copy — same pipeline as preview
  const renderHighQualityCanvas = useCallback(
    async (annotations: Annotation[], _imagePath?: string): Promise<HTMLCanvasElement | null> => {
      if (!screenshotImage) return null;

      try {
        const bgSrc = getBackgroundImageSrc(settings);
        let bgImage: HTMLImageElement | null = null;
        if (bgSrc) {
          bgImage = await loadImage(bgSrc);
        }

        const canvas = renderFullCanvas(
          screenshotImage,
          settings,
          { top: paddingTop, bottom: paddingBottom, left: paddingLeft, right: paddingRight },
          bgImage,
          { renderEffects: true }
        );

        if (annotations.length > 0) {
          const ctx = canvas.getContext("2d");
          if (ctx) {
            const MAX_PREVIEW_DIM = 1400;
            const previewScale = Math.min(1.0, MAX_PREVIEW_DIM / Math.max(canvas.width, canvas.height));
            const previewWidth = Math.round(canvas.width * previewScale);
            const exportScaleFactor = canvas.width / previewWidth;

            ctx.save();
            if (exportScaleFactor !== 1) {
              ctx.scale(exportScaleFactor, exportScaleFactor);
            }

            const nonText = annotations.filter((a) => a.type !== "text");
            const textAnns = annotations.filter((a) => a.type === "text");
            nonText.forEach((annotation) => {
              drawAnnotationOnCanvas(ctx, annotation);
            });
            textAnns.forEach((annotation) => {
              drawAnnotationOnCanvas(ctx, annotation);
            });

            ctx.restore();
          }
        }

        return canvas;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(`Failed to render high-quality image: ${message}`);
        return null;
      }
    },
    [screenshotImage, settings, paddingTop, paddingBottom, paddingLeft, paddingRight]
  );

  return {
    previewUrl,
    isGenerating,
    error,
    renderHighQualityCanvas,
  };
}
