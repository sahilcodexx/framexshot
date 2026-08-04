/**
 * Returns the image URL directly so native <img> elements render via browser engine
 * with loading="lazy" and decoding="async", eliminating dynamic canvas rendering overhead.
 */
export function getThumbnailUrl(src: string, _maxDim: number = 140): string {
  return src;
}
