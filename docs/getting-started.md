# Getting started

Run **Cast** on your machine.

## 1. Install

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

## 2. Installation

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

## 3. Run

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

## 4. Development

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

## 5. Prerequisites

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

## Next

- Full detail: [Guide](/guide)
- Source: [github.com/mediuswareltd/cast](https://github.com/mediuswareltd/cast)
