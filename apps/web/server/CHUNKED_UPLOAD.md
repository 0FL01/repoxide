# Chunked Upload API

## Overview

The Repomix server supports chunked file uploads for large ZIP files. Files are split into chunks on the client side and uploaded sequentially to avoid limitations on file upload size (e.g., Cloudflare's 100MB limit).

## Configuration

- **Max file size**: 50MB
- **Chunk size**: 1MB (recommended for compatibility with Cloudflare)
- **Upload TTL**: 1 hour (session expires after creation)
- **Max concurrent uploads**: 100

## Endpoints

### 1. Initialize Upload

**POST** `/api/upload/init`

Creates a new upload session and returns an upload ID.

**Request Body:**
```json
{
  "fileName": "my-repository.zip",
  "fileSize": 10485760,
  "totalChunks": 10
}
```

**Response:**
```json
{
  "uploadId": "550e8400-e29b-41d4-a716-446655440000",
  "expiresIn": 3600
}
```

**Validation:**
- `fileName` must end with `.zip`
- `fileSize` must not exceed 50MB
- `totalChunks` must be greater than 0

### 2. Upload Chunk

**POST** `/api/upload/chunk?uploadId={id}&chunkIndex={index}`

Uploads a single chunk of data. The chunk data should be sent as the raw request body.

**Query Parameters:**
- `uploadId` (UUID): Session ID from initialization
- `chunkIndex` (number): Zero-based chunk index (0 to totalChunks-1)

**Request Body:** Binary data (the chunk)

**Response:**
```json
{
  "uploadId": "550e8400-e29b-41d4-a716-446655440000",
  "chunksReceived": 5,
  "totalChunks": 10,
  "complete": false
}
```

**Features:**
- Idempotent: Uploading the same chunk twice is safe
- Chunks can be uploaded in any order
- Session expires 1 hour after initialization

### 3. Check Upload Status

**GET** `/api/upload/status/{uploadId}`

Returns the current status of an upload session.

**Response:**
```json
{
  "uploadId": "550e8400-e29b-41d4-a716-446655440000",
  "chunksReceived": 5,
  "totalChunks": 10,
  "progress": 0.5,
  "complete": false
}
```

### 4. Process Upload

Once all chunks are uploaded, use the `uploadId` with the pack endpoint:

**POST** `/api/pack`

```
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="uploadId"

550e8400-e29b-41d4-a716-446655440000
--boundary
Content-Disposition: form-data; name="format"

markdown
--boundary--
```

The server will:
1. Verify all chunks are received
2. Assemble chunks into a complete ZIP file
3. Extract and process the ZIP
4. Return the packed repository

## Client Example

```javascript
async function uploadFileInChunks(file, chunkSize = 1024 * 1024) {
  const totalChunks = Math.ceil(file.size / chunkSize);
  
  // 1. Initialize upload
  const initResponse = await fetch('/api/upload/init', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      fileName: file.name,
      fileSize: file.size,
      totalChunks
    })
  });
  const { uploadId } = await initResponse.json();
  
  // 2. Upload chunks
  for (let i = 0; i < totalChunks; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, file.size);
    const chunk = file.slice(start, end);
    
    await fetch(`/api/upload/chunk?uploadId=${uploadId}&chunkIndex=${i}`, {
      method: 'POST',
      body: chunk
    });
  }
  
  // 3. Process the upload
  const formData = new FormData();
  formData.append('uploadId', uploadId);
  formData.append('format', 'markdown');
  
  const packResponse = await fetch('/api/pack', {
    method: 'POST',
    body: formData
  });
  
  return await packResponse.json();
}
```

## Error Handling

### Common Errors

- **400 Bad Request**
  - Invalid file name (must be .zip)
  - Invalid chunk index
  - Upload incomplete
  
- **404 Not Found**
  - Upload session not found
  
- **410 Gone**
  - Upload session expired
  
- **413 Payload Too Large**
  - File size exceeds 50MB
  
- **503 Service Unavailable**
  - Too many concurrent uploads

## Security Features

1. **ZIP Extraction Safety:**
   - Path traversal protection
   - File count limit (50,000 files)
   - Uncompressed size limit (2GB)
   - Compression ratio check (protect against ZIP bombs)
   - Path length and nesting depth limits

2. **Session Management:**
   - Automatic cleanup of expired sessions
   - Temporary directory isolation
   - Background cleanup task runs every 60 seconds

3. **Concurrency Control:**
   - Max 100 concurrent upload sessions
   - Per-session chunk tracking
   - Thread-safe state management (RwLock)

## Implementation Details

### State Management

Upload sessions are stored in memory using `RwLock<HashMap<Uuid, UploadSession>>`. Each session tracks:
- File name and size
- Total chunks and received chunks
- Temporary directory path
- Creation and expiration timestamps

### Chunk Storage

Chunks are stored as individual files in a temporary directory:
```
/tmp/repomix-upload-{uuid}/
  chunk_000000
  chunk_000001
  chunk_000002
  ...
```

### Assembly Process

When all chunks are received:
1. Verify chunk count matches expected total
2. Read chunks in order (0 to N-1)
3. Concatenate into a single file
4. Verify file size matches expected size
5. Process as a regular ZIP upload

### Cleanup

Sessions are automatically cleaned up when:
- They expire (1 hour after creation)
- All chunks are successfully assembled and processed
- Background cleanup task detects expired sessions
