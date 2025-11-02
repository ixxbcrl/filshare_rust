# Quick Start Guide

Get your file transfer backend running in minutes!

## Prerequisites

- Rust installed (check with `rustc --version`)
- Git (optional, for version control)

## Steps

### 1. Navigate to Project Directory

```bash
cd fileshare_rust
```

### 2. Create Environment File (Optional)

```bash
cp .env.example .env
```

The defaults work out of the box:
- Database: `sqlite:./files.db`
- Upload Directory: `./uploads`
- Port: `3000`

### 3. Run the Application

**Development mode (with debug info):**
```bash
cargo run
```

**Production mode (optimized):**
```bash
cargo run --release
```

### 4. Test the API

Open your browser or use curl:

**Health check:**
```bash
curl http://localhost:3000/health
```

**Upload a file:**
```bash
curl -X POST http://localhost:3000/api/files \
  -F "file=@/path/to/your/file.txt" \
  -F "description=My first upload"
```

**List all files:**
```bash
curl http://localhost:3000/api/files
```

## What's Running?

Your server is now:
- ✅ Listening on `http://0.0.0.0:3000`
- ✅ Accepting file uploads
- ✅ Storing files in `./uploads/`
- ✅ Tracking metadata in SQLite database
- ✅ Ready for React frontend connections

## Next Steps

1. **Build React Frontend**: See `API_DOCUMENTATION.md` for complete API details and React examples
2. **Deploy to Raspberry Pi**: Follow the deployment guide in `README.md`
3. **Connect via Tailscale**: Install Tailscale to access from anywhere

## Troubleshooting

**Port already in use?**
```bash
# Change port in .env file
echo "PORT=8080" >> .env
cargo run
```

**Want to see debug logs?**
```bash
RUST_LOG=debug cargo run
```

**Database locked?**
```bash
# Make sure no other instance is running
pkill fileshare_rust  # or use Task Manager on Windows
```

## Project Structure

```
fileshare_rust/
├── src/              # Rust source code
├── uploads/          # Uploaded files (created automatically)
├── files.db          # SQLite database (created automatically)
├── Cargo.toml        # Dependencies
└── .env              # Configuration (optional)
```

That's it! Your file transfer backend is ready to use.

For complete documentation, see:
- `README.md` - Full setup and deployment guide
- `API_DOCUMENTATION.md` - Complete API reference with React examples
