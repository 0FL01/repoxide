import type { Context, Next } from 'hono';
import { rateLimiter } from '../domains/pack/utils/sharedInstance.js';
import { getClientInfo } from '../utils/clientInfo.js';
import { createErrorResponse } from '../utils/http.js';

export function rateLimitMiddleware() {
  return async function rateLimitMiddleware(c: Context, next: Next) {
    const clientInfo = getClientInfo(c);

    // Check rate limit
    if (!rateLimiter.isAllowed(clientInfo.ip)) {
      const remainingTime = Math.ceil(rateLimiter.getRemainingTime(clientInfo.ip) / 1000);
      const message = `Rate limit exceeded.\nPlease try again in ${remainingTime} seconds.`;
      const requestId = c.get('requestId');
      return c.json(createErrorResponse(message, requestId), 429);
    }

    await next();
  };
}
