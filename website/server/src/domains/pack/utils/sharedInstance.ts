import type { PackResult } from '../../../types.js';
import { RateLimiter } from '../../../utils/rateLimit.js';
import { RequestCache } from './cache.js';

// Create shared instances
export const cache = new RequestCache<PackResult>(180); // 3 minutes cache
export const rateLimiter = new RateLimiter(60_000, process.env.NODE_ENV === 'development' ? 1000 : 300); // 1000 requests per minute in dev, 300 in production
