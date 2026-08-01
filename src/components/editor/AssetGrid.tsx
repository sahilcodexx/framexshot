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
  const [activeCategory, setActiveCategory] = useState(categories[0]?.name || "");

  const currentCategory = categories.find((cat) => cat.name === activeCategory);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-foreground font-mono text-balance">Wallpapers</h3>
      </div>
      
      {categories.length > 1 && (
        <div className="flex p-1 bg-secondary/50 rounded-lg border border-border/50">
          {categories.map((category) => (
            <button
              key={category.name}
              onClick={() => setActiveCategory(category.name)}
              aria-label={`Select ${category.name} category`}
              className={cn(
                "flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-colors",
                activeCategory === category.name
                  ? "bg-muted text-white shadow-sm"
                  : "text-muted-foreground hover:text-foreground hover:bg-muted/30"
              )}
            >
              {category.name === "Wallpapers" ? "Wallpapers" : category.name === "Mac Assets" ? "Mac" : category.name}
            </button>
          ))}
        </div>
      )}

      <div className="grid grid-cols-4 gap-2 max-h-[400px] overflow-y-auto pr-4 pb-2 [&::-webkit-scrollbar]:w-1.5 [&::-webkit-scrollbar-thumb]:bg-muted [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-track]:bg-transparent">
        {currentCategory?.assets.map((asset) => (
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
