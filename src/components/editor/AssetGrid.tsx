import { useState, useEffect, memo } from "react";
import { cn } from "@/lib/utils";
import { getThumbnailUrl } from "@/lib/thumbnail-utils";

export interface Asset {
  id: string;
  src: string;
  name: string;
}

export interface AssetCategory {
  name: string;
  assets: Asset[];
}

interface AssetGridProps {
  categories: AssetCategory[];
  selectedImage: string | null;
  backgroundType: string;
  onImageSelect: (imageSrc: string) => void;
}

const ThumbnailButton = memo(function ThumbnailButton({
  asset,
  isSelected,
  onSelect,
}: {
  asset: Asset;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const [thumbSrc, setThumbSrc] = useState<string>(asset.src);

  useEffect(() => {
    let isMounted = true;
    getThumbnailUrl(asset.src, 140).then((url) => {
      if (isMounted) setThumbSrc(url);
    });
    return () => {
      isMounted = false;
    };
  }, [asset.src]);

  return (
    <button
      onClick={onSelect}
      aria-label={`Select ${asset.name} background`}
      className={cn(
        "group relative w-full aspect-square rounded-lg overflow-hidden transition-shadow",
        isSelected
          ? "ring-2 ring-blue-500 ring-offset-2 ring-offset-card"
          : "ring-1 ring-border hover:ring-ring"
      )}
    >
      <img
        src={thumbSrc}
        alt={asset.name}
        loading="lazy"
        decoding="async"
        className="w-full h-full object-cover transition-transform group-hover:scale-110"
      />
      {isSelected && (
        <div className="absolute inset-0 bg-blue-500/20 flex items-center justify-center">
          <div className="size-6 rounded-full bg-blue-500 flex items-center justify-center shadow-lg">
            <svg className="size-3.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
            </svg>
          </div>
        </div>
      )}
    </button>
  );
});

export const AssetGrid = memo(function AssetGrid({ categories, selectedImage, backgroundType, onImageSelect }: AssetGridProps) {
  const allAssets = categories.flatMap((cat) => cat.assets);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-foreground font-mono text-balance">Wallpapers</h3>
      </div>

      <div className="grid grid-cols-4 gap-2 pb-2">
        {allAssets.map((asset) => (
          <ThumbnailButton
            key={asset.id}
            asset={asset}
            isSelected={backgroundType === "image" && selectedImage === asset.src}
            onSelect={() => onImageSelect(asset.src)}
          />
        ))}
      </div>
    </div>
  );
});
