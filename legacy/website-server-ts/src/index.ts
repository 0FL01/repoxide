import { serve } from '@hono/node-server';
import { Hono } from 'hono';
import { bodyLimit } from 'hono/body-limit';
import { compress } from 'hono/compress';
import { timeout } from 'hono/timeout';
import { packAction } from './actions/packAction.js';
import { initUploadAction, uploadChunkAction, uploadStatusAction } from './actions/uploadActions.js';
import { bodyLimitMiddleware } from './middlewares/bodyLimit.js';
import { cloudLoggerMiddleware } from './middlewares/cloudLogger.js';
import { corsMiddleware } from './middlewares/cors.js';
import { rateLimitMiddleware } from './middlewares/rateLimit.js';
import { CHUNK_UPLOAD_CONFIG } from './domains/pack/chunkedUpload.js';
import { logInfo, logMemoryUsage } from './utils/logger.js';
import { getProcessConcurrency } from './utils/processConcurrency.js';

const API_TIMEOUT_MS = 600_000; // 600 seconds (10 minutes)

// Log server metrics on startup
logInfo('Server starting', {
  metrics: {
    processConcurrency: getProcessConcurrency(),
  },
});

// Log initial memory usage
logMemoryUsage('Server startup', {
  processConcurrency: getProcessConcurrency(),
});

const app = new Hono();

// Configure CORS
app.use('/*', corsMiddleware);

// Enable compression
app.use(compress());

// Set timeout for API routes
app.use('/api', timeout(API_TIMEOUT_MS));

// Setup custom logger
app.use('*', cloudLoggerMiddleware());

// Apply rate limiting to API routes
app.use('/api/*', rateLimitMiddleware());

// Health check endpoint
app.get('/health', (c) => c.text('OK'));

// Main packing endpoint
app.post('/api/pack', bodyLimitMiddleware, packAction);

// Chunked upload endpoints
// Smaller body limit for chunks (2MB to have some margin over 1MB chunks)
const chunkBodyLimit = bodyLimit({
  maxSize: CHUNK_UPLOAD_CONFIG.CHUNK_SIZE + 512 * 1024, // chunk size + 512KB margin
  onError: (c) => {
    return c.json({ error: 'Chunk size too large' }, 413);
  },
});

app.post('/api/upload/init', initUploadAction);
app.post('/api/upload/chunk', chunkBodyLimit, uploadChunkAction);
app.get('/api/upload/status/:uploadId', uploadStatusAction);

// Start server
const port = process.env.PORT ? Number.parseInt(process.env.PORT, 10) : 3000;
logInfo(`Server starting on port ${port}`);

serve({
  fetch: app.fetch,
  port,
});

// Export app for testing
export default app;
