import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { AppError } from '../../utils/errorHandler.js';
import { logInfo, logError } from '../../utils/logger.js';

// Chunked upload configuration
export const CHUNK_UPLOAD_CONFIG = {
    CHUNK_SIZE: 1 * 1024 * 1024, // 1MB per chunk (safe for Cloudflare)
    MAX_FILE_SIZE: 50 * 1024 * 1024, // 50MB total
    UPLOAD_TIMEOUT: 5 * 60 * 1000, // 5 minutes to complete upload
    MAX_CONCURRENT_UPLOADS: 100, // Maximum concurrent uploads
} as const;

interface UploadSession {
    id: string;
    fileName: string;
    fileSize: number;
    totalChunks: number;
    receivedChunks: Set<number>;
    tempDir: string;
    createdAt: number;
    expiresAt: number;
}

// In-memory store for upload sessions
const uploadSessions = new Map<string, UploadSession>();

// Cleanup expired sessions periodically
setInterval(() => {
    const now = Date.now();
    for (const [id, session] of uploadSessions.entries()) {
        if (now > session.expiresAt) {
            cleanupSession(id).catch((err) => {
                logError('Failed to cleanup expired session', err instanceof Error ? err : new Error(String(err)), { uploadId: id });
            });
        }
    }
}, 60_000); // Check every minute

/**
 * Initialize a chunked upload session
 */
export async function initChunkedUpload(
    fileName: string,
    fileSize: number,
    chunkSize: number = CHUNK_UPLOAD_CONFIG.CHUNK_SIZE
): Promise<{ uploadId: string; chunkSize: number; totalChunks: number }> {
    // Validate file size
    if (fileSize > CHUNK_UPLOAD_CONFIG.MAX_FILE_SIZE) {
        throw new AppError(
            `File size ${(fileSize / 1024 / 1024).toFixed(2)}MB exceeds maximum limit of ${CHUNK_UPLOAD_CONFIG.MAX_FILE_SIZE / 1024 / 1024}MB`,
            413
        );
    }

    // Validate file name
    if (!fileName || !fileName.endsWith('.zip')) {
        throw new AppError('Only ZIP files are allowed', 400);
    }

    // Check concurrent uploads limit
    if (uploadSessions.size >= CHUNK_UPLOAD_CONFIG.MAX_CONCURRENT_UPLOADS) {
        throw new AppError('Too many concurrent uploads. Please try again later.', 503);
    }

    const uploadId = randomUUID();
    const totalChunks = Math.ceil(fileSize / chunkSize);
    const tempDir = path.join(os.tmpdir(), `repomix-upload-${uploadId}`);

    await fs.mkdir(tempDir, { recursive: true });

    const session: UploadSession = {
        id: uploadId,
        fileName,
        fileSize,
        totalChunks,
        receivedChunks: new Set(),
        tempDir,
        createdAt: Date.now(),
        expiresAt: Date.now() + CHUNK_UPLOAD_CONFIG.UPLOAD_TIMEOUT,
    };

    uploadSessions.set(uploadId, session);

    logInfo('Chunked upload initialized', {
        uploadId,
        fileName,
        fileSize,
        totalChunks,
        chunkSize,
    });

    return {
        uploadId,
        chunkSize,
        totalChunks,
    };
}

/**
 * Receive a chunk of the file
 */
export async function receiveChunk(
    uploadId: string,
    chunkIndex: number,
    chunkData: ArrayBuffer
): Promise<{ received: number; total: number; isComplete: boolean }> {
    const session = uploadSessions.get(uploadId);

    if (!session) {
        throw new AppError('Upload session not found or expired', 404);
    }

    // Check if session expired
    if (Date.now() > session.expiresAt) {
        await cleanupSession(uploadId);
        throw new AppError('Upload session expired', 410);
    }

    // Validate chunk index
    if (chunkIndex < 0 || chunkIndex >= session.totalChunks) {
        throw new AppError(`Invalid chunk index: ${chunkIndex}. Expected 0-${session.totalChunks - 1}`, 400);
    }

    // Skip if already received
    if (session.receivedChunks.has(chunkIndex)) {
        return {
            received: session.receivedChunks.size,
            total: session.totalChunks,
            isComplete: session.receivedChunks.size === session.totalChunks,
        };
    }

    // Write chunk to temp file
    const chunkPath = path.join(session.tempDir, `chunk_${chunkIndex.toString().padStart(6, '0')}`);
    await fs.writeFile(chunkPath, Buffer.from(chunkData));

    session.receivedChunks.add(chunkIndex);

    const isComplete = session.receivedChunks.size === session.totalChunks;

    logInfo('Chunk received', {
        uploadId,
        chunkIndex,
        received: session.receivedChunks.size,
        total: session.totalChunks,
        isComplete,
    });

    return {
        received: session.receivedChunks.size,
        total: session.totalChunks,
        isComplete,
    };
}

/**
 * Assemble chunks into a complete file and return it as a File object
 */
export async function assembleFile(uploadId: string): Promise<File> {
    const session = uploadSessions.get(uploadId);

    if (!session) {
        throw new AppError('Upload session not found or expired', 404);
    }

    // Check if all chunks received
    if (session.receivedChunks.size !== session.totalChunks) {
        throw new AppError(
            `Upload incomplete. Received ${session.receivedChunks.size} of ${session.totalChunks} chunks`,
            400
        );
    }

    logInfo('Assembling file from chunks', {
        uploadId,
        totalChunks: session.totalChunks,
        fileName: session.fileName,
    });

    // Read all chunks and concatenate
    const chunks: Buffer[] = [];
    for (let i = 0; i < session.totalChunks; i++) {
        const chunkPath = path.join(session.tempDir, `chunk_${i.toString().padStart(6, '0')}`);
        const chunkData = await fs.readFile(chunkPath);
        chunks.push(chunkData);
    }

    const fileBuffer = Buffer.concat(chunks);

    // Verify file size
    if (fileBuffer.length !== session.fileSize) {
        throw new AppError(
            `File size mismatch. Expected ${session.fileSize}, got ${fileBuffer.length}`,
            400
        );
    }

    // Create a File object from the buffer
    const file = new File([fileBuffer], session.fileName, { type: 'application/zip' });

    // Cleanup session after successful assembly
    await cleanupSession(uploadId);

    logInfo('File assembled successfully', {
        uploadId,
        fileName: session.fileName,
        fileSize: fileBuffer.length,
    });

    return file;
}

/**
 * Get upload session status
 */
export function getUploadStatus(uploadId: string): {
    found: boolean;
    received?: number;
    total?: number;
    isComplete?: boolean;
    expiresIn?: number;
} {
    const session = uploadSessions.get(uploadId);

    if (!session) {
        return { found: false };
    }

    return {
        found: true,
        received: session.receivedChunks.size,
        total: session.totalChunks,
        isComplete: session.receivedChunks.size === session.totalChunks,
        expiresIn: Math.max(0, session.expiresAt - Date.now()),
    };
}

/**
 * Cleanup an upload session
 */
export async function cleanupSession(uploadId: string): Promise<void> {
    const session = uploadSessions.get(uploadId);

    if (!session) {
        return;
    }

    try {
        await fs.rm(session.tempDir, { recursive: true, force: true });
    } catch (err) {
        logError('Failed to cleanup temp directory', err instanceof Error ? err : new Error(String(err)), {
            uploadId,
            tempDir: session.tempDir,
        });
    }

    uploadSessions.delete(uploadId);

    logInfo('Upload session cleaned up', { uploadId });
}
