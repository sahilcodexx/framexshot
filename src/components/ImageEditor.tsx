import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "sonner";
import { Loader2, Redo2, Undo2, Palette, Image as ImageIcon, LayoutGrid, Box } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { BackgroundSelector, gradientOptions } from "./editor/BackgroundSelector";
import { AssetGrid } from "./editor/AssetGrid";
import { EffectsPanel } from "./editor/EffectsPanel";
import { MockupSelector } from "./editor/MockupSelector";
import { StyleSelector } from "./editor/StyleSelector";
import { LayoutPresets } from "./editor/LayoutPresets";
import { BorderPresets } from "./editor/BorderPresets";
import { ShadowPresets } from "./editor/ShadowPresets";
import { ImagePositionControl } from "./editor/ImagePositionControl";
import { AnnotationToolbar } from "./editor/AnnotationToolbar";
import { AnnotationCanvas } from "./editor/AnnotationCanvas";
import { PropertiesPanel } from "./editor/PropertiesPanel";
import { Annotation, ToolType } from "@/types/annotations";
import { usePreviewGenerator } from "@/hooks/usePreviewGenerator";
import { getAssetCategories } from "@/lib/asset-registry";
const assetCategories = getAssetCategories();
import {
  useEditorStore,
  useBackgroundType,
  useBlurAmount,
  useAnnotations,
  useCanUndo,
  useCanRedo,
  useSelectedImageSrc,
  useGradientId,
  useWindowFrame,
  useFrameStyle,
  useLayoutPreset,
  useBorderPreset,
  useShadowPreset,
  useShowMockup,
  useNoiseAmount,
  useBorderRadius,
  usePaddingTop,
  usePaddingBottom,
  usePaddingLeft,
  usePaddingRight,
  useShadowBlur,
  useShadowOffsetX,
  useShadowOffsetY,
  useShadowOpacity,
  useFramePadding,
  useFrameOpacity,
  useImageScale,
  useImageOffsetX,
  useImageOffsetY,
  editorActions,
} from "@/stores";
import type { EditorSettings } from "@/stores/editorStore";

interface ImageEditorProps {
  imagePath: string;
  onSave: (editedImageData: string) => void;
  onCancel: () => void;
}

export function ImageEditor({ imagePath, onSave, onCancel }: ImageEditorProps) {
  // Granular selectors to minimize re-renders
  const backgroundType = useBackgroundType();
  const blurAmount = useBlurAmount();
  const annotations = useAnnotations();
  const canUndo = useCanUndo();
  const canRedo = useCanRedo();
  const selectedImageSrc = useSelectedImageSrc();
  const gradientId = useGradientId();
  const windowFrame = useWindowFrame();
  const frameStyle = useFrameStyle();
  const layoutPreset = useLayoutPreset();
  const borderPreset = useBorderPreset();
  const shadowPreset = useShadowPreset();
  const showMockup = useShowMockup();
  const noiseAmount = useNoiseAmount();
  const borderRadius = useBorderRadius();
  const paddingTop = usePaddingTop();
  const paddingBottom = usePaddingBottom();
  const paddingLeft = usePaddingLeft();
  const paddingRight = usePaddingRight();
  const shadowBlur = useShadowBlur();
  const shadowOffsetX = useShadowOffsetX();
  const shadowOffsetY = useShadowOffsetY();
  const shadowOpacity = useShadowOpacity();
  const framePadding = useFramePadding();
  const frameOpacity = useFrameOpacity();
  const imageScale = useImageScale();
  const imageOffsetX = useImageOffsetX();
  const imageOffsetY = useImageOffsetY();
  const shadow = useMemo(() => ({ blur: shadowBlur, offsetX: shadowOffsetX, offsetY: shadowOffsetY, opacity: shadowOpacity }), [shadowBlur, shadowOffsetX, shadowOffsetY, shadowOpacity]);
  const customColor = useEditorStore((s: { settings: { customColor: string } }) => s.settings.customColor);

  // Memoized settings object for hooks that need the full shape
  const settings = useMemo(() => ({
    backgroundType: backgroundType as EditorSettings['backgroundType'],
    customColor,
    selectedImageSrc,
    gradientId,
    gradientSrc: '',
    gradientColors: ['#667eea', '#764ba2'] as [string, string],
    blurAmount,
    noiseAmount,
    borderRadius,
    paddingTop,
    paddingBottom,
    paddingLeft,
    paddingRight,
    shadow,
    windowFrame,
    frameStyle,
    layoutPreset,
    borderPreset,
    shadowPreset,
    showMockup,
    framePadding,
    frameOpacity,
    imageScale,
    imageOffsetX,
    imageOffsetY,
  }), [backgroundType, customColor, selectedImageSrc, gradientId, blurAmount, noiseAmount, borderRadius, paddingTop, paddingBottom, paddingLeft, paddingRight, shadow, windowFrame, frameStyle, layoutPreset, borderPreset, shadowPreset, showMockup, framePadding, frameOpacity, imageScale, imageOffsetX, imageOffsetY]);

  // Use stable actions object (not a hook, doesn't cause re-renders)
  const actions = editorActions;
  
  // Screenshot image state
  const [screenshotImage, setScreenshotImage] = useState<HTMLImageElement | null>(null);
  const [imageLoaded, setImageLoaded] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  
  // Save/copy state
  const [isSaving, setIsSaving] = useState(false);
  const [isCopying, setIsCopying] = useState(false);
  const [tempDir, setTempDir] = useState<string>("/private/tmp");

   // Annotation UI state (not part of undo/redo)
  const [selectedTool, setSelectedTool] = useState<ToolType>("select");
  const [selectedAnnotation, setSelectedAnnotation] = useState<Annotation | null>(null);
  
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Preview generator hook
  const { previewUrl, error: previewError, renderHighQualityCanvas } = usePreviewGenerator({
    screenshotImage,
    settings,
    canvasRef,
    paddingTop: settings.paddingTop,
    paddingBottom: settings.paddingBottom,
    paddingLeft: settings.paddingLeft,
    paddingRight: settings.paddingRight,
    imagePath,
  });

  // Combined error
  const error = loadError || previewError;

  // Initialize store once on mount
  useEffect(() => {
    editorActions.initialize();
  }, []);

  // Restore window state on mount (keep decorations OFF — we use custom TitleBar)
  useEffect(() => {
    const restoreWindowState = async () => {
      try {
        const appWindow = getCurrentWindow();
        await Promise.all([
          appWindow.setFullscreen(false),
          appWindow.setAlwaysOnTop(false),
        ]);
        // Intentionally NOT calling setDecorations(true) — custom TitleBar handles chrome
      } catch (err) {
        console.error("Failed to restore window state:", err);
      }
    };
    restoreWindowState();

    // Get the system temp directory
    invoke<string>("get_temp_directory")
      .then((dir) => setTempDir(dir))
      .catch((err) => console.error("Failed to get temp directory:", err));
  }, []);

  // Load main screenshot image
  useEffect(() => {
    setLoadError(null);
    setImageLoaded(false);
    setScreenshotImage(null);

    if (!imagePath) {
      setLoadError("No image path provided");
      return;
    }

    const img = new Image();
    img.onload = () => {
      setScreenshotImage(img);
      setImageLoaded(true);

      // Calculate smart default padding: 10% of average dimension, capped at 400px
      const avgDimension = (img.width + img.height) / 2;
      const defaultPadding = Math.min(Math.round(avgDimension * 0.1), 400);
      actions.setPaddingTopTransient(defaultPadding);
      actions.setPaddingBottomTransient(defaultPadding);
      actions.setPaddingLeftTransient(defaultPadding);
      actions.setPaddingRightTransient(defaultPadding);
    };
    img.onerror = () => {
      setLoadError(`Failed to load image from: ${imagePath}`);
    };

    const assetUrl = convertFileSrc(imagePath);
    img.crossOrigin = "anonymous";
    img.src = assetUrl;

    return () => {
      img.onload = null;
      img.onerror = null;
    };
  }, [imagePath, actions]);

  // Save handler
  const handleSave = useCallback(async () => {
    if (!screenshotImage || isSaving || isCopying) return;
    
    setIsSaving(true);
    try {
      const highQualityCanvas = await renderHighQualityCanvas(annotations, imagePath);
      
      if (!highQualityCanvas) {
        setIsSaving(false);
        return;
      }

      highQualityCanvas.toBlob(
        (blob) => {
          if (blob) {
            const reader = new FileReader();
            reader.onloadend = () => {
              onSave(reader.result as string);
              setIsSaving(false);
            };
            reader.onerror = () => {
              setLoadError("Failed to read image data");
              setIsSaving(false);
            };
            reader.readAsDataURL(blob);
          } else {
            setIsSaving(false);
          }
        },
        "image/png",
        1.0
      );
    } catch (err) {
      setLoadError(`Failed to save: ${err instanceof Error ? err.message : String(err)}`);
      setIsSaving(false);
    }
  }, [screenshotImage, annotations, renderHighQualityCanvas, onSave, isSaving, isCopying, imagePath]);

  // Copy handler
  const handleCopy = useCallback(async () => {
    if (!screenshotImage || isSaving || isCopying) return;
    
    setIsCopying(true);
    try {
      const highQualityCanvas = await renderHighQualityCanvas(annotations, imagePath);
      
      if (!highQualityCanvas) {
        setIsCopying(false);
        return;
      }

      const dataUrl = highQualityCanvas.toDataURL("image/png");
      
      await invoke<string>("save_edited_image", {
        imageData: dataUrl,
        saveDir: tempDir,
        copyToClip: true,
      });
      
      toast.success("Screenshot copied to clipboard!", {
        duration: 2000,
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setLoadError(`Failed to copy: ${errorMessage}`);
      toast.error("Failed to copy", {
        description: errorMessage,
        duration: 3000,
      });
    } finally {
      setIsCopying(false);
    }
  }, [screenshotImage, annotations, renderHighQualityCanvas, isSaving, isCopying, tempDir, imagePath]);

  // Annotation handlers
  const handleAnnotationAdd = useCallback((annotation: Annotation) => {
    actions.addAnnotation(annotation);
    setSelectedAnnotation(annotation);
    if (annotation.type !== "number") {
      setSelectedTool("select");
    }
  }, [actions]);

  const handleAnnotationUpdate = useCallback((annotation: Annotation) => {
    actions.updateAnnotation(annotation);
    setSelectedAnnotation(annotation);
  }, [actions]);

  const handleAnnotationDelete = useCallback((id: string) => {
    actions.deleteAnnotation(id);
    setSelectedAnnotation((prev) => prev?.id === id ? null : prev);
  }, [actions]);

  const handleDeleteSelected = useCallback(() => {
    if (selectedAnnotation) {
      handleAnnotationDelete(selectedAnnotation.id);
    }
  }, [selectedAnnotation, handleAnnotationDelete]);

  // Undo/Redo handlers
  const handleUndo = useCallback(() => {
    actions.undo();
    setSelectedAnnotation(null);
  }, [actions]);

  const handleRedo = useCallback(() => {
    actions.redo();
    setSelectedAnnotation(null);
  }, [actions]);

  const handleResetPadding = useCallback(() => {
    if (!screenshotImage) return;
    const avgDimension = (screenshotImage.width + screenshotImage.height) / 2;
    const defaultPadding = Math.min(Math.round(avgDimension * 0.1), 400);
    actions.setPaddingTop(defaultPadding);
    actions.setPaddingBottom(defaultPadding);
    actions.setPaddingLeft(defaultPadding);
    actions.setPaddingRight(defaultPadding);
  }, [actions, screenshotImage]);

  // Delete annotation with keyboard
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }
      
      if (e.key === "Delete" || e.key === "Backspace") {
        if (selectedAnnotation) {
          e.preventDefault();
          handleAnnotationDelete(selectedAnnotation.id);
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedAnnotation, handleAnnotationDelete]);

  // Keyboard shortcuts for save/copy/undo/redo
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if typing in input fields
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      // Save: Cmd+S
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        if (imageLoaded && !isSaving && !isCopying) {
          handleSave();
        }
      }
      // Copy: Cmd+Shift+C
      if ((e.metaKey || e.ctrlKey) && e.key === "c" && e.shiftKey) {
        e.preventDefault();
        if (imageLoaded && !isSaving && !isCopying) {
          handleCopy();
        }
      }
      // Undo: Cmd+Z
      if ((e.metaKey || e.ctrlKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        handleUndo();
      }
      // Redo: Cmd+Shift+Z or Cmd+Y
      if ((e.metaKey || e.ctrlKey) && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
        e.preventDefault();
        handleRedo();
      }
      // Cancel: Escape
      if (e.key === "Escape") {
        onCancel();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [imageLoaded, isSaving, isCopying, handleSave, handleCopy, handleUndo, handleRedo, onCancel]);

  // Find selected gradient for BackgroundSelector
  const selectedGradientOption = gradientOptions.find(g => g.id === settings.gradientId) || gradientOptions[0];

  // Right sidebar tab state for ultra-smooth performance
  const [activeTab, setActiveTab] = useState<"bg" | "wallpapers" | "frame" | "layout">("bg");

  return (
    <div className="flex flex-col w-full h-full bg-[#181818] text-foreground font-sans select-none">
      <div className="flex flex-1 min-h-0">
        {/* Left Sidebar — Actions, Image Transform, Properties, Shadow & Effects */}
        <div className="w-[280px] xl:w-[320px] shrink-0 border-r border-[#252525] bg-[#1a1a1a] flex flex-col sidebar-scroll transition-all duration-200">
          {/* Action buttons bar */}
          <div className="p-3 flex gap-2 border-b border-[#252525] sticky top-0 bg-[#1a1a1a] z-20">
            <Button 
              variant="ghost"
              onClick={onCancel}
              className="flex-1 h-8 rounded-md bg-[#252525] hover:bg-[#2e2e2e] text-xs font-medium border border-[#333]"
            >
              Cancel
            </Button>
            <Button 
              variant="ghost"
              onClick={handleCopy} 
              disabled={!imageLoaded || isSaving || isCopying}
              className="flex-1 h-8 rounded-md bg-[#252525] hover:bg-[#2e2e2e] text-xs font-medium border border-[#333] disabled:opacity-50"
            >
              {isCopying ? <Loader2 className="size-3.5 animate-spin" /> : "Copy"}
            </Button>
            <Button 
              variant="default"
              onClick={handleSave} 
              disabled={!imageLoaded || isSaving || isCopying}
              className="flex-1 h-8 rounded-md bg-white text-black hover:bg-white/90 text-xs font-medium disabled:opacity-50"
            >
              {isSaving ? <Loader2 className="size-3.5 animate-spin" /> : "Export"}
            </Button>
          </div>

          <div className="p-4 space-y-6 pb-10">
            {/* Image Transform */}
            <div className="space-y-2.5">
              <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Image</h3>
              <ImagePositionControl
                imageScale={settings.imageScale ?? 1.0}
                imageOffsetX={settings.imageOffsetX ?? 0}
                imageOffsetY={settings.imageOffsetY ?? 0}
                onScaleChangeTransient={actions.setImageScaleTransient}
                onScaleChange={actions.setImageScale}
                onOffsetTransient={actions.setImageOffsetTransient}
                onOffsetCommit={actions.setImageOffset}
                onReset={actions.resetImageTransform}
              />
            </div>

            {/* Properties */}
            <div className="space-y-2.5">
              <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Properties</h3>
              {selectedAnnotation ? (
                <PropertiesPanel annotation={selectedAnnotation} onUpdate={handleAnnotationUpdate} />
              ) : (
                <div className="text-xs text-muted-foreground text-center py-5 bg-[#202020] rounded-lg border border-[#2e2e2e]">
                  Select an annotation to edit
                </div>
              )}
            </div>

            {/* Shadow */}
            <div className="space-y-2.5">
              <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Shadow</h3>
              <ShadowPresets
                shadowPreset={settings.shadowPreset || "spread"}
                opacity={settings.shadow.opacity}
                showMockup={settings.showMockup !== false}
                onPresetChange={actions.setShadowPreset}
                onOpacityChangeTransient={actions.setShadowOpacityTransient}
                onOpacityChange={actions.setShadowOpacity}
                onToggleMockup={() => actions.setShowMockup(!(settings.showMockup !== false))}
              >
                {settings.shadowPreset !== "none" && (
                  <div className="space-y-3 pt-1">
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-xs text-muted-foreground font-medium">Blur</label>
                        <span className="text-xs text-muted-foreground font-mono tabular-nums">{settings.shadow.blur}px</span>
                      </div>
                      <EffectsPanel
                        blurAmount={settings.blurAmount}
                        noiseAmount={settings.noiseAmount}
                        paddingTop={settings.paddingTop}
                        paddingBottom={settings.paddingBottom}
                        paddingLeft={settings.paddingLeft}
                        paddingRight={settings.paddingRight}
                        shadow={settings.shadow}
                        onBlurAmountChangeTransient={actions.setBlurAmountTransient}
                        onNoiseChangeTransient={actions.setNoiseAmountTransient}
                        onPaddingTopChangeTransient={actions.setPaddingTopTransient}
                        onPaddingBottomChangeTransient={actions.setPaddingBottomTransient}
                        onPaddingLeftChangeTransient={actions.setPaddingLeftTransient}
                        onPaddingRightChangeTransient={actions.setPaddingRightTransient}
                        onAllPaddingChangeTransient={actions.setAllPaddingTransient}
                        onShadowBlurChangeTransient={actions.setShadowBlurTransient}
                        onShadowOffsetXChangeTransient={actions.setShadowOffsetXTransient}
                        onShadowOffsetYChangeTransient={actions.setShadowOffsetYTransient}
                        onShadowOpacityChangeTransient={actions.setShadowOpacityTransient}
                        onBlurAmountChange={actions.setBlurAmount}
                        onNoiseChange={actions.setNoiseAmount}
                        onPaddingTopChange={actions.setPaddingTop}
                        onPaddingBottomChange={actions.setPaddingBottom}
                        onPaddingLeftChange={actions.setPaddingLeft}
                        onPaddingRightChange={actions.setPaddingRight}
                        onAllPaddingChange={actions.setAllPadding}
                        onShadowBlurChange={actions.setShadowBlur}
                        onShadowOffsetXChange={actions.setShadowOffsetX}
                        onShadowOffsetYChange={actions.setShadowOffsetY}
                        onShadowOpacityChange={actions.setShadowOpacity}
                        onSaveAsDefaults={actions.saveEffectSettingsAsDefaults}
                        onResetPadding={handleResetPadding}
                        shadowOnly
                      />
                    </div>
                  </div>
                )}
              </ShadowPresets>
            </div>

            {/* Effects */}
            <div className="space-y-2.5">
              <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Effects</h3>
              <EffectsPanel
                blurAmount={settings.blurAmount}
                noiseAmount={settings.noiseAmount}
                paddingTop={settings.paddingTop}
                paddingBottom={settings.paddingBottom}
                paddingLeft={settings.paddingLeft}
                paddingRight={settings.paddingRight}
                shadow={settings.shadow}
                onBlurAmountChangeTransient={actions.setBlurAmountTransient}
                onNoiseChangeTransient={actions.setNoiseAmountTransient}
                onPaddingTopChangeTransient={actions.setPaddingTopTransient}
                onPaddingBottomChangeTransient={actions.setPaddingBottomTransient}
                onPaddingLeftChangeTransient={actions.setPaddingLeftTransient}
                onPaddingRightChangeTransient={actions.setPaddingRightTransient}
                onAllPaddingChangeTransient={actions.setAllPaddingTransient}
                onShadowBlurChangeTransient={actions.setShadowBlurTransient}
                onShadowOffsetXChangeTransient={actions.setShadowOffsetXTransient}
                onShadowOffsetYChangeTransient={actions.setShadowOffsetYTransient}
                onShadowOpacityChangeTransient={actions.setShadowOpacityTransient}
                onBlurAmountChange={actions.setBlurAmount}
                onNoiseChange={actions.setNoiseAmount}
                onPaddingTopChange={actions.setPaddingTop}
                onPaddingBottomChange={actions.setPaddingBottom}
                onPaddingLeftChange={actions.setPaddingLeft}
                onPaddingRightChange={actions.setPaddingRight}
                onAllPaddingChange={actions.setAllPadding}
                onShadowBlurChange={actions.setShadowBlur}
                onShadowOffsetXChange={actions.setShadowOffsetX}
                onShadowOffsetYChange={actions.setShadowOffsetY}
                onShadowOpacityChange={actions.setShadowOpacity}
                onSaveAsDefaults={actions.saveEffectSettingsAsDefaults}
                onResetPadding={handleResetPadding}
                hideShadow
              />
            </div>
          </div>
        </div>

        {/* Center Canvas Area */}
        <div className="flex-1 flex flex-col relative overflow-hidden bg-[#111111]">
          {/* Floating Top Toolbar */}
          <div className="absolute top-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-1 bg-[#141414]/90 backdrop-blur-xl p-1.5 rounded-full border border-[#2a2a2a] shadow-2xl">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleUndo}
                    disabled={!canUndo}
                    className="size-8 rounded-full text-muted-foreground hover:text-white disabled:opacity-30"
                  >
                    <Undo2 className="size-4" aria-hidden="true" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom" className="text-xs bg-[#1a1a1a] border-[#2a2a2a]">Undo ⌘Z</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleRedo}
                    disabled={!canRedo}
                    className="size-8 rounded-full text-muted-foreground hover:text-white disabled:opacity-30"
                  >
                    <Redo2 className="size-4" aria-hidden="true" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom" className="text-xs bg-[#1a1a1a] border-[#2a2a2a]">Redo ⌘⇧Z</TooltipContent>
              </Tooltip>
            </TooltipProvider>

            <div className="w-[1px] h-4 bg-[#333] mx-2" />

            <AnnotationToolbar
              selectedTool={selectedTool}
              onToolSelect={setSelectedTool}
              onDelete={selectedAnnotation ? handleDeleteSelected : undefined}
            />
          </div>

          {/* Main Canvas */}
          <div className="flex-1 flex items-center justify-center p-4 sm:p-6 md:p-8 lg:p-12 overflow-hidden min-w-0 min-h-0 relative">
            <div className="w-full h-full flex items-center justify-center min-w-0 min-h-0 z-10">
              {previewUrl ? (
                <AnnotationCanvas
                  annotations={annotations}
                  selectedAnnotation={selectedAnnotation}
                  selectedTool={selectedTool}
                  previewUrl={previewUrl}
                  showTransparencyGrid={settings.backgroundType === "transparent"}
                  onAnnotationAdd={handleAnnotationAdd}
                  onAnnotationUpdate={handleAnnotationUpdate}
                  onAnnotationSelect={setSelectedAnnotation}
                  onAnnotationDelete={handleAnnotationDelete}
                  onToolSelect={setSelectedTool}
                />
              ) : imageLoaded ? (
                <div className="text-muted-foreground text-sm">Generating preview...</div>
              ) : error ? (
                <div className="text-center text-red-400 p-5">
                  <p className="mb-1 text-sm font-medium">Could not load image</p>
                  <small className="text-xs opacity-70">{error}</small>
                </div>
              ) : (
                <div className="text-muted-foreground text-sm flex items-center gap-2">
                  <Loader2 className="size-4 animate-spin" />
                  Loading image...
                </div>
              )}
              <canvas ref={canvasRef} style={{ display: "none" }} />
            </div>
          </div>
        </div>

        {/* Right Sidebar — Redesigned High-Performance Tabbed Styling Panel */}
        <div className="w-[270px] xl:w-[320px] shrink-0 border-l border-[#252525] bg-[#1a1a1a] flex flex-col sidebar-scroll transition-all duration-200">
          {/* Header & Tabs */}
          <div className="p-3 border-b border-[#252525] sticky top-0 bg-[#1a1a1a] z-20 space-y-2">
            <div className="flex items-center justify-between px-1">
              <span className="text-xs font-semibold text-white tracking-tight flex items-center gap-1.5">
                <div className="size-3.5 rounded-sm bg-gradient-to-br from-blue-500 to-purple-600" />
                Appearance & Customization
              </span>
            </div>

            {/* Segmented Tab Controls */}
            <div className="grid grid-cols-4 p-1 bg-[#141414] rounded-xl border border-[#262626]">
              <button
                type="button"
                onClick={() => setActiveTab("bg")}
                className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg text-xs transition-all duration-150 ${
                  activeTab === "bg"
                    ? "bg-[#262626] text-white shadow-sm font-medium border border-white/10"
                    : "text-muted-foreground hover:text-white hover:bg-[#1c1c1c]"
                }`}
                title="Background"
              >
                <Palette className="size-3.5 shrink-0" />
                <span>Bg</span>
              </button>

              <button
                type="button"
                onClick={() => setActiveTab("wallpapers")}
                className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg text-xs transition-all duration-150 ${
                  activeTab === "wallpapers"
                    ? "bg-[#262626] text-white shadow-sm font-medium border border-white/10"
                    : "text-muted-foreground hover:text-white hover:bg-[#1c1c1c]"
                }`}
                title="Media & Wallpapers"
              >
                <ImageIcon className="size-3.5 shrink-0" />
                <span>Media</span>
              </button>

              <button
                type="button"
                onClick={() => setActiveTab("frame")}
                className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg text-xs transition-all duration-150 ${
                  activeTab === "frame"
                    ? "bg-[#262626] text-white shadow-sm font-medium border border-white/10"
                    : "text-muted-foreground hover:text-white hover:bg-[#1c1c1c]"
                }`}
                title="Style, Frame & Border"
              >
                <LayoutGrid className="size-3.5 shrink-0" />
                <span>Style</span>
              </button>

              <button
                type="button"
                onClick={() => setActiveTab("layout")}
                className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-lg text-xs transition-all duration-150 ${
                  activeTab === "layout"
                    ? "bg-[#262626] text-white shadow-sm font-medium border border-white/10"
                    : "text-muted-foreground hover:text-white hover:bg-[#1c1c1c]"
                }`}
                title="Layout & Presets"
              >
                <Box className="size-3.5 shrink-0" />
                <span>Layout</span>
              </button>
            </div>
          </div>

          {/* Tab Content */}
          <div className="p-4 space-y-6 pb-10">
            {activeTab === "bg" && (
              <div className="space-y-4">
                <BackgroundSelector
                  backgroundType={settings.backgroundType as "transparent" | "white" | "black" | "gray" | "gradient" | "custom" | "image"}
                  customColor={settings.customColor}
                  selectedGradient={selectedGradientOption.id}
                  onBackgroundTypeChange={actions.setBackgroundType}
                  onCustomColorChange={actions.setCustomColor}
                  onGradientSelect={actions.setGradient}
                  onImageSelect={actions.handleImageSelect}
                />
              </div>
            )}

            {activeTab === "wallpapers" && (
              <div className="space-y-4">
                <AssetGrid
                  categories={assetCategories}
                  selectedImage={settings.selectedImageSrc}
                  backgroundType={settings.backgroundType}
                  onImageSelect={actions.handleImageSelect}
                />
              </div>
            )}

            {activeTab === "frame" && (
              <div className="space-y-6">
                <div className="space-y-2.5">
                  <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Style</h3>
                  <StyleSelector
                    frameStyle={settings.frameStyle || "default"}
                    framePadding={settings.framePadding ?? 0}
                    frameOpacity={settings.frameOpacity ?? 100}
                    onChange={actions.setFrameStyle}
                    onFramePaddingChangeTransient={actions.setFramePaddingTransient}
                    onFramePaddingChange={actions.setFramePadding}
                    onFrameOpacityChangeTransient={actions.setFrameOpacityTransient}
                    onFrameOpacityChange={actions.setFrameOpacity}
                  />
                </div>

                <div className="space-y-2.5">
                  <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Mockup Frame</h3>
                  <MockupSelector
                    windowFrame={settings.windowFrame || "none"}
                    onChange={actions.setWindowFrame}
                  />
                </div>

                <div className="space-y-2.5">
                  <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Border</h3>
                  <BorderPresets
                    borderPreset={settings.borderPreset || "curved"}
                    borderRadius={settings.borderRadius}
                    onPresetChange={actions.setBorderPreset}
                    onBorderRadiusChangeTransient={actions.setBorderRadiusTransient}
                    onBorderRadiusChange={actions.setBorderRadius}
                  />
                </div>
              </div>
            )}

            {activeTab === "layout" && (
              <div className="space-y-3">
                <h3 className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">3D Perspective Presets</h3>
                <LayoutPresets
                  layoutPreset={settings.layoutPreset || "flat"}
                  onChange={actions.setLayoutPreset}
                  previewUrl={previewUrl}
                />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
