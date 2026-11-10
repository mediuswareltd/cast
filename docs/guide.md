# Guide

A lightweight Rust service for fast video/audio format conversion with automatic file cleanup and no persistent storage.

](https://mediuswareltd.github.io/cast/)

A lightweight, privacy-focused video converter service built in Rust. This API provides fast, efficient video format conversion with automatic file cleanup.

## Features

- Multiple format conversions: MOV→MP4, WEBM→MP4, MP4→MOV, MP4→MP3
- File size limits: 200MB for video, 100MB for audio
- Automatic file cleanup (5-10 minutes)
- Files deleted immediately after download
- RESTful API with CORS support
- Built with Rust for performance and reliability
- Privacy-focused: no persistent storage

## Prerequisites

Before running this application, you need to install:

1. **Rust** (latest stable version)
   ```bash
   # Windows (using rustup)
   # Download and run: https://rustup.rs/
   
   # Linux/macOS
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **FFmpeg** (required for video conversion)
   ```bash
   # Windows (using Chocolatey)
   choco install ffmpeg
   
   # Windows (using Scoop)
   scoop install ffmpeg
   
   # macOS
   brew install ffmpeg
   
   # Ubuntu/Debian
   sudo apt update && sudo apt install ffmpeg
   
   # Verify installation
   ffmpeg -version
   ```

## Installation

1. Clone or extract the project:
   ```bash
   cd farruk-video-converter
   ```

2. Copy the environment file:
   ```bash
   # Windows (PowerShell)
   copy .env.example .env
   
   # Linux/macOS
   cp .env.example .env
   ```

3. (Optional) Edit `.env` to configure server settings:
   ```env
   BIND_ADDRESS=127.0.0.1
   PORT=8080
   RUST_LOG=info
   ```

## Running the Application

### Development Mode

```bash
cargo run
```

### Production Mode (Optimized)

```bash
cargo build --release
./target/release/video-converter

# Windows
.\target\release\video-converter.exe
```

The server will start at `http://127.0.0.1:8080` (or your configured address).

## API Endpoints

### 1. Health Check
```http
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-14T12:00:00Z"
}
```

### 2. Upload and Convert File
```http
POST /api/upload
Content-Type: multipart/form-data
```

**Parameters:**
- `file` (file): The video file to convert
- `format` (string): Target format (mp4, mov, mp3)

**Example using cURL:**
```bash
curl -X POST http://localhost:8080/api/upload \
  -F "file=@video.mov" \
  -F "format=mp4"
```

**Success Response:**
```json
{
  "success": true,
  "message": "Conversion successful",
  "file_id": "550e8400-e29b-41d4-a716-446655440000",
  "filename": "video_550e8400-e29b-41d4-a716-446655440000.mp4"
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "File size exceeds maximum limit of 200 MB"
}
```

### 3. Download Converted File
```http
GET /api/download/{filename}
```

**Example:**
```bash
curl -O http://localhost:8080/api/download/video_550e8400-e29b-41d4-a716-446655440000.mp4
```

**Note:** Files are automatically deleted after download.

## Supported Conversions

| From   | To    | Max Size | Description           |
|--------|-------|----------|-----------------------|
| MOV    | MP4   | 200 MB   | Video conversion      |
| WEBM   | MP4   | 200 MB   | Video conversion      |
| MP4    | MOV   | 200 MB   | Video conversion      |
| MP4    | MP3   | 100 MB   | Audio extraction      |

## File Cleanup Policy

- **Immediate deletion**: Files are deleted right after download
- **Automatic cleanup**: Any remaining files are deleted after 10 minutes
- **Background service**: Cleanup runs every 5 minutes
- **No persistent storage**: All conversions are temporary

## Integration with Node.js/Next.js

### Frontend Example (Next.js)

```typescript
// app/api/convert/route.ts
export async function POST(request: Request) {
  const formData = await request.formData();
  
  const response = await fetch('http://localhost:8080/api/upload', {
    method: 'POST',
    body: formData,
  });
  
  return Response.json(await response.json());
}

// components/VideoConverter.tsx
'use client';

export default function VideoConverter() {
  const [file, setFile] = useState<File | null>(null);
  const [format, setFormat] = useState('mp4');
  const [loading, setLoading] = useState(false);
  
  const handleConvert = async () => {
    if (!file) return;
    
    setLoading(true);
    const formData = new FormData();
    formData.append('file', file);
    formData.append('format', format);
    
    try {
      const response = await fetch('/api/convert', {
        method: 'POST',
        body: formData,
      });
      
      const data = await response.json();
      
      if (data.success) {
        // Download the converted file
        window.location.href = `http://localhost:8080/api/download/${data.filename}`;
      }
    } catch (error) {
      console.error('Conversion failed:', error);
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <div className="converter">
      <input 
        type="file" 
        accept="video/*"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
      />
      <select value={format} onChange={(e) => setFormat(e.target.value)}>
        <option value="mp4">MP4</option>
        <option value="mov">MOV</option>
        <option value="mp3">MP3 (Audio Only)</option>
      </select>
      <button onClick={handleConvert} disabled={!file || loading}>
        {loading ? 'Converting...' : 'Convert'}
      </button>
    </div>
  );
}
```

### Backend Proxy (Optional - Node.js/Express)

```javascript
// server.js
const express = require('express');
const multer = require('multer');
const axios = require('axios');
const FormData = require('form-data');

const app = express();
const upload = multer();

app.post('/api/convert', upload.single('file'), async (req, res) => {
  try {
    const formData = new FormData();
    formData.append('file', req.file.buffer, req.file.originalname);
    formData.append('format', req.body.format);
    
    const response = await axios.post('http://localhost:8080/api/upload', formData, {
      headers: formData.getHeaders(),
    });
    
    res.json(response.data);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});
```

## Configuration

### Environment Variables

- `BIND_ADDRESS`: Server IP address (default: `127.0.0.1`)
- `PORT`: Server port (default: `8080`)
- `RUST_LOG`: Log level (`error`, `warn`, `info`, `debug`, `trace`)

### Constants (in `src/main.rs`)

```rust
const VIDEO_MAX_SIZE: usize = 200 * 1024 * 1024; // 200 MB
const AUDIO_MAX_SIZE: usize = 100 * 1024 * 1024; // 100 MB
const CLEANUP_INTERVAL_MINUTES: u64 = 5;
const FILE_RETENTION_MINUTES: u64 = 10;
```

## Architecture

```
video-converter/
├── src/
│   ├── main.rs          # API server and routes
│   ├── conversion.rs    # FFmpeg conversion logic
│   ├── validation.rs    # File validation
│   └── cleanup.rs       # Auto-cleanup service
├── temp/
│   ├── uploads/         # Temporary uploaded files
│   └── outputs/         # Converted files (auto-deleted)
├── Cargo.toml           # Dependencies
├── .env                 # Configuration
└── README.md
```

## Error Handling

The API provides clear error messages for common issues:

- **File too large**: Exceeds size limits
- **Invalid format**: Unsupported conversion
- **Missing file**: No file uploaded
- **FFmpeg error**: Conversion failed
- **File not found**: Already deleted or doesn't exist

## Performance

- **Lightweight**: Rust provides minimal memory footprint
- **Fast**: Native performance for I/O operations
- **Concurrent**: Handles multiple conversions simultaneously
- **Efficient**: FFmpeg with optimized settings

## Security & Privacy

- No database or persistent storage
- Files automatically deleted
- Sanitized filenames prevent path traversal
- File size limits prevent abuse
- CORS enabled for frontend integration
- Validation of all inputs

## Troubleshooting

### FFmpeg not found
```
Error: FFmpeg is not installed or not in PATH!
```
**Solution**: Install FFmpeg and ensure it's in your system PATH.

### Port already in use
```
Error: Address already in use
```
**Solution**: Change the `PORT` in `.env` or stop the process using port 8080.

### File size exceeded
```
Error: File size exceeds maximum limit
```
**Solution**: Ensure your file is under 200MB (video) or 100MB (audio).

### Unsupported conversion
```
Error: Unsupported conversion: AVI to MP4
```
**Solution**: Only use supported conversions listed above.

## Development

### Run tests
```bash
cargo test
```

### Check code
```bash
cargo clippy
```

### Format code
```bash
cargo fmt
```

## Documentation

Full docs site (VitePress): **https://mediuswareltd.github.io/cast/**

Local preview:

```bash
cd docs
npm install
npm run docs:dev
```


---

Repository: [mediuswareltd/cast](https://github.com/mediuswareltd/cast)
