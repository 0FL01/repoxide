import type { PackOptions } from '../../../types.js';

// Cache configuration
const MAX_CACHE_SIZE = 50; // Maximum number of entries in cache (LRU limit)
const DEFAULT_TTL_SECONDS = 300; // 5 minutes TTL

interface CacheEntry<T> {
  value: string; // JSON string (no compression for CPU optimization)
  timestamp: number;
}

export class RequestCache<T> {
  private cache: Map<string, CacheEntry<T>> = new Map();
  private readonly ttl: number;
  private readonly maxSize: number;

  constructor(ttlInSeconds = DEFAULT_TTL_SECONDS, maxSize = MAX_CACHE_SIZE) {
    this.ttl = ttlInSeconds * 1000;
    this.maxSize = maxSize;

    // Set up periodic cache cleanup
    setInterval(() => this.cleanup(), ttlInSeconds * 1000);
  }

  get(key: string): T | undefined {
    const entry = this.cache.get(key);
    if (!entry) {
      return undefined;
    }

    const now = Date.now();
    if (now - entry.timestamp > this.ttl) {
      this.cache.delete(key);
      return undefined;
    }

    try {
      // Parse JSON directly (no decompression needed)
      return JSON.parse(entry.value);
    } catch (error) {
      console.error('Error parsing cache entry:', error);
      this.cache.delete(key);
      return undefined;
    }
  }

  set(key: string, value: T): void {
    // Evict oldest entries if cache is full
    while (this.cache.size >= this.maxSize) {
      this.evictOldest();
    }

    // Store as JSON string (no compression for CPU optimization)
    this.cache.set(key, {
      value: JSON.stringify(value),
      timestamp: Date.now(),
    });
  }

  // Remove the oldest entry (LRU eviction)
  private evictOldest(): void {
    let oldestKey: string | null = null;
    let oldestTimestamp = Number.POSITIVE_INFINITY;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.timestamp < oldestTimestamp) {
        oldestTimestamp = entry.timestamp;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.cache.delete(oldestKey);
    }
  }

  // Remove expired entries from cache
  cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > this.ttl) {
        this.cache.delete(key);
      }
    }
  }

  // Get current cache size (for monitoring)
  get size(): number {
    return this.cache.size;
  }
}

// Cache key generation utility
export function generateCacheKey(
  identifier: string,
  format: string,
  options: PackOptions,
  type: 'url' | 'file',
): string {
  return JSON.stringify({
    identifier,
    format,
    options,
    type,
  });
}
