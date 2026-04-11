export interface PackOptions {
  removeComments: boolean;
  removeEmptyLines: boolean;
  showLineNumbers: boolean;
  fileSummary?: boolean;
  directoryStructure?: boolean;
  includePatterns?: string;
  ignorePatterns?: string;
  outputParsable?: boolean;
  compress?: boolean;
}

export interface FileInfo {
  path: string;
  charCount: number;
  tokenCount: number;
  selected?: boolean;
}

export interface PackRequest {
  url: string;
  format: 'xml' | 'markdown' | 'plain';
  options: PackOptions;
  signal?: AbortSignal;
  file?: File;
  onUploadProgress?: (progress: number) => void;
}

export interface PackResult {
  content: string;
  format: string;
  metadata: {
    repository: string;
    timestamp: string;
    summary: {
      totalFiles: number;
      totalCharacters: number;
      totalTokens: number;
    };
    topFiles: {
      path: string;
      charCount: number;
      tokenCount: number;
    }[];
    allFiles?: FileInfo[];
  };
}

export interface ErrorResponse {
  error: string;
}

export class ApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

// Chunked upload configuration - should match server config
const CHUNK_UPLOAD_CONFIG = {
  CHUNK_SIZE: 1 * 1024 * 1024, // 1MB per chunk
  CHUNKED_UPLOAD_THRESHOLD: 2 * 1024 * 1024, // Use chunked upload for files > 2MB
} as const;

interface InitUploadResponse {
  uploadId: string;
  expiresIn: number;
}

interface ChunkUploadResponse {
  uploadId: string;
  chunksReceived: number;
  totalChunks: number;
  complete: boolean;
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';

/**
 * Initialize a chunked upload session
 */
async function initChunkedUpload(
  fileName: string,
  fileSize: number,
  signal?: AbortSignal
): Promise<InitUploadResponse> {
  // Calculate total chunks
  const totalChunks = Math.ceil(fileSize / CHUNK_UPLOAD_CONFIG.CHUNK_SIZE);

  const response = await fetch(`${API_BASE_URL}/api/upload/init`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ fileName, fileSize, totalChunks }),
    signal,
  });

  const data = await response.json();

  if (!response.ok) {
    throw new ApiError((data as ErrorResponse).error || 'Failed to initialize upload');
  }

  return data as InitUploadResponse;
}

/**
 * Upload a single chunk
 */
async function uploadChunk(
  uploadId: string,
  chunkIndex: number,
  chunkData: ArrayBuffer,
  signal?: AbortSignal
): Promise<ChunkUploadResponse> {
  const response = await fetch(
    `${API_BASE_URL}/api/upload/chunk?uploadId=${encodeURIComponent(uploadId)}&chunkIndex=${chunkIndex}`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/octet-stream',
      },
      body: chunkData,
      signal,
    }
  );

  const data = await response.json();

  if (!response.ok) {
    throw new ApiError((data as ErrorResponse).error || 'Failed to upload chunk');
  }

  return data as ChunkUploadResponse;
}

/**
 * Upload a file using chunked upload
 */
async function chunkedUpload(
  file: File,
  signal?: AbortSignal,
  onProgress?: (progress: number) => void
): Promise<string> {
  // Calculate total chunks locally
  const totalChunks = Math.ceil(file.size / CHUNK_UPLOAD_CONFIG.CHUNK_SIZE);

  // Initialize upload session
  const initResponse = await initChunkedUpload(file.name, file.size, signal);
  const { uploadId } = initResponse;

  // Upload chunks sequentially
  for (let i = 0; i < totalChunks; i++) {
    // Check if aborted
    if (signal?.aborted) {
      throw new ApiError('Upload cancelled');
    }

    const start = i * CHUNK_UPLOAD_CONFIG.CHUNK_SIZE;
    const end = Math.min(start + CHUNK_UPLOAD_CONFIG.CHUNK_SIZE, file.size);
    const chunkBlob = file.slice(start, end);
    const chunkData = await chunkBlob.arrayBuffer();

    await uploadChunk(uploadId, i, chunkData, signal);

    // Report progress
    if (onProgress) {
      onProgress(((i + 1) / totalChunks) * 100);
    }
  }

  return uploadId;
}

/**
 * Check if a file should use chunked upload
 */
function shouldUseChunkedUpload(file: File): boolean {
  return file.size > CHUNK_UPLOAD_CONFIG.CHUNKED_UPLOAD_THRESHOLD;
}

/**
 * Pack a repository using either direct upload or chunked upload
 */
export async function packRepository(request: PackRequest): Promise<PackResult> {
  const formData = new FormData();

  if (request.file) {
    if (shouldUseChunkedUpload(request.file)) {
      // Use chunked upload for large files
      const uploadId = await chunkedUpload(
        request.file,
        request.signal,
        request.onUploadProgress
      );
      formData.append('uploadId', uploadId);
    } else {
      // Direct file upload for small files
      formData.append('file', request.file);
    }
  } else {
    formData.append('url', request.url);
  }

  formData.append('format', request.format);
  formData.append('options', JSON.stringify(request.options));

  const response = await fetch(`${API_BASE_URL}/api/pack`, {
    method: 'POST',
    body: formData,
    signal: request.signal,
  });

  const data = await response.json();

  if (!response.ok) {
    throw new ApiError((data as ErrorResponse).error);
  }

  return data as PackResult;
}
