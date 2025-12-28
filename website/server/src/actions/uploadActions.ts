import type { Context } from 'hono';
import { z } from 'zod';
import {
    initChunkedUpload,
    receiveChunk,
    getUploadStatus,
    CHUNK_UPLOAD_CONFIG,
} from '../domains/pack/chunkedUpload.js';
import { createErrorResponse } from '../utils/http.js';
import { logError, logInfo } from '../utils/logger.js';
import { validateRequest } from '../utils/validation.js';

// Schema for upload initialization
const initUploadSchema = z.object({
    fileName: z
        .string()
        .min(1, 'File name is required')
        .max(255, 'File name is too long')
        .refine((name) => name.endsWith('.zip'), { message: 'Only ZIP files are allowed' }),
    fileSize: z
        .number()
        .int()
        .positive('File size must be positive')
        .max(CHUNK_UPLOAD_CONFIG.MAX_FILE_SIZE, `File size must be less than ${CHUNK_UPLOAD_CONFIG.MAX_FILE_SIZE / 1024 / 1024}MB`),
});

// Schema for chunk upload
const uploadChunkSchema = z.object({
    uploadId: z.string().uuid('Invalid upload ID'),
    chunkIndex: z.number().int().min(0, 'Chunk index must be non-negative'),
});

/**
 * Initialize a chunked upload session
 * POST /api/upload/init
 */
export const initUploadAction = async (c: Context) => {
    try {
        const requestId = c.get('requestId');
        const body = await c.req.json();

        const validatedData = validateRequest(initUploadSchema, body);

        const result = await initChunkedUpload(
            validatedData.fileName,
            validatedData.fileSize,
            CHUNK_UPLOAD_CONFIG.CHUNK_SIZE
        );

        logInfo('Upload session initialized', {
            requestId,
            uploadId: result.uploadId,
            fileName: validatedData.fileName,
            fileSize: validatedData.fileSize,
        });

        return c.json({
            uploadId: result.uploadId,
            chunkSize: result.chunkSize,
            totalChunks: result.totalChunks,
        });
    } catch (error) {
        logError('Failed to initialize upload', error instanceof Error ? error : new Error('Unknown error'), {
            requestId: c.get('requestId'),
        });

        const { handlePackError } = await import('../utils/errorHandler.js');
        const appError = handlePackError(error);
        return c.json(createErrorResponse(appError.message, c.get('requestId')), appError.statusCode);
    }
};

/**
 * Upload a chunk
 * POST /api/upload/chunk
 */
export const uploadChunkAction = async (c: Context) => {
    try {
        const requestId = c.get('requestId');

        // Get upload ID and chunk index from query params or headers
        const uploadId = c.req.query('uploadId') || c.req.header('X-Upload-Id');
        const chunkIndexStr = c.req.query('chunkIndex') || c.req.header('X-Chunk-Index');

        if (!uploadId || !chunkIndexStr) {
            return c.json(
                createErrorResponse('Missing uploadId or chunkIndex', requestId),
                400
            );
        }

        const chunkIndex = Number.parseInt(chunkIndexStr, 10);
        if (Number.isNaN(chunkIndex)) {
            return c.json(
                createErrorResponse('Invalid chunkIndex', requestId),
                400
            );
        }

        const validatedData = validateRequest(uploadChunkSchema, { uploadId, chunkIndex });

        // Get chunk data from request body
        const arrayBuffer = await c.req.arrayBuffer();

        if (!arrayBuffer || arrayBuffer.byteLength === 0) {
            return c.json(
                createErrorResponse('Chunk data is required', requestId),
                400
            );
        }

        const result = await receiveChunk(
            validatedData.uploadId,
            validatedData.chunkIndex,
            arrayBuffer
        );

        return c.json({
            received: result.received,
            total: result.total,
            isComplete: result.isComplete,
        });
    } catch (error) {
        logError('Failed to upload chunk', error instanceof Error ? error : new Error('Unknown error'), {
            requestId: c.get('requestId'),
        });

        const { handlePackError } = await import('../utils/errorHandler.js');
        const appError = handlePackError(error);
        return c.json(createErrorResponse(appError.message, c.get('requestId')), appError.statusCode);
    }
};

/**
 * Get upload status
 * GET /api/upload/status/:uploadId
 */
export const uploadStatusAction = async (c: Context) => {
    try {
        const uploadId = c.req.param('uploadId');

        if (!uploadId) {
            return c.json(
                createErrorResponse('Upload ID is required', c.get('requestId')),
                400
            );
        }

        const status = getUploadStatus(uploadId);

        if (!status.found) {
            return c.json(
                createErrorResponse('Upload session not found', c.get('requestId')),
                404
            );
        }

        return c.json({
            received: status.received,
            total: status.total,
            isComplete: status.isComplete,
            expiresIn: status.expiresIn,
        });
    } catch (error) {
        logError('Failed to get upload status', error instanceof Error ? error : new Error('Unknown error'), {
            requestId: c.get('requestId'),
        });

        const { handlePackError } = await import('../utils/errorHandler.js');
        const appError = handlePackError(error);
        return c.json(createErrorResponse(appError.message, c.get('requestId')), appError.statusCode);
    }
};
