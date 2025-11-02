# File Transfer Backend - Rust

A robust file transfer backend API built with Rust, designed to run on Raspberry Pi with Tailscale. Features file upload, download, listing, and deletion with SQLite database storage.

## Features

- **File Upload**: Upload files with optional descriptions via multipart form data
- **File Download**: Download files with original filenames preserved
- **File Listing**: View all files with metadata (size, type, upload date, etc.)
- **File Deletion**: Delete files from both filesystem and database
- **SQLite Database**: Persistent metadata storage
- **CORS Enabled**: Ready for React frontend integration
- **UUID-based Storage**: Prevents filename conflicts
- **Comprehensive Logging**: Debug and trace capabilities

## Project Structure

```
fileshare_rust/
├── src/
│   ├── main.rs          # Application entry point and server setup
│   ├── db.rs            # Database connection and initialization
│   ├── models.rs        # Data models and response structures
│   ├── storage.rs       # File storage service
│   └── handlers.rs      # HTTP request handlers
├── migrations/
│   └── 001_create_files_table.sql  # Database schema
├── uploads/             # File storage directory (created automatically)
├── Cargo.toml          # Rust dependencies
├── .env.example        # Environment variables template
├── API_DOCUMENTATION.md # Complete API documentation for frontend
└── README.md           # This file
```

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- SQLite (usually pre-installed on Raspberry Pi OS)

## Installation

### 1. Clone or Copy the Project

Navigate to your desired directory and ensure the project files are in place.

### 2. Install Rust (if not already installed)

On Raspberry Pi:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Configure Environment Variables

Copy the example environment file and customize as needed:
```bash
cp .env.example .env
```

Edit `.env`:
```bash
DATABASE_URL=sqlite:./files.db
UPLOAD_DIR=./uploads
PORT=3000
```

### 4. Build the Project

For development:
```bash
cargo build
```

For production (optimized):
```bash
cargo build --release
```

## Running the Application

### Development Mode

```bash
cargo run
```

### Production Mode

```bash
cargo run --release
```

Or run the compiled binary directly:
```bash
./target/release/fileshare_rust
```

The server will start on `http://0.0.0.0:3000` by default.

## Deployment on Raspberry Pi with Tailscale

### 1. Install Tailscale on Raspberry Pi

```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
```

### 2. Get Your Tailscale IP

```bash
tailscale ip -4
```

This will show your Tailscale IPv4 address (e.g., `100.x.x.x`).

### 3. Build and Run the Application

```bash
cd /path/to/fileshare_rust
cargo build --release
./target/release/fileshare_rust
```

### 4. Run as a Background Service (Recommended)

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/fileshare.service
```

Add the following content (adjust paths as needed):

```ini
[Unit]
Description=File Transfer Rust Backend
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/fileshare_rust
Environment="DATABASE_URL=sqlite:./files.db"
Environment="UPLOAD_DIR=./uploads"
Environment="PORT=3000"
ExecStart=/home/pi/fileshare_rust/target/release/fileshare_rust
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable fileshare.service
sudo systemctl start fileshare.service
```

Check status:
```bash
sudo systemctl status fileshare.service
```

View logs:
```bash
sudo journalctl -u fileshare.service -f
```

### 5. Access from Other Devices

From any device connected to your Tailscale network:
```
http://100.x.x.x:3000
```

Replace `100.x.x.x` with your Raspberry Pi's Tailscale IP address.

## API Endpoints

Base URL: `http://localhost:3000` (or your Tailscale IP)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/files` | Upload a file |
| GET | `/api/files` | List all files |
| GET | `/api/files/:id` | Get file metadata |
| GET | `/api/files/:id/download` | Download a file |
| DELETE | `/api/files/:id` | Delete a file |

For detailed API documentation with React examples, see [API_DOCUMENTATION.md](./API_DOCUMENTATION.md).

## Testing the API

### Using curl

**Upload a file:**
```bash
curl -X POST http://localhost:3000/api/files \
  -F "file=@/path/to/your/file.pdf" \
  -F "description=Test file"
```

**List all files:**
```bash
curl http://localhost:3000/api/files
```

**Download a file:**
```bash
curl -O -J http://localhost:3000/api/files/{file-id}/download
```

**Delete a file:**
```bash
curl -X DELETE http://localhost:3000/api/files/{file-id}
```

### Using a Web Browser

1. Navigate to `http://localhost:3000/health` to check if the server is running
2. Use tools like Postman or Insomnia for more complex testing
3. Build the React frontend to get a full user interface

## Configuration Options

### Environment Variables

- `DATABASE_URL`: SQLite database path (default: `sqlite:./files.db`)
- `UPLOAD_DIR`: Directory for storing uploaded files (default: `./uploads`)
- `PORT`: Server port (default: `3000`)

### CORS Configuration

The application allows all origins by default. For production, edit `src/main.rs`:

```rust
let cors = CorsLayer::new()
    .allow_origin("https://your-frontend-domain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::DELETE])
    .allow_headers(Any);
```

## Development

### Project Dependencies

Main dependencies (see `Cargo.toml`):
- **axum**: Web framework
- **tokio**: Async runtime
- **sqlx**: Async SQLite driver
- **tower-http**: CORS and middleware
- **serde**: Serialization/deserialization
- **uuid**: Unique file identifiers
- **chrono**: Timestamp handling

### Adding New Features

1. Add new models in `src/models.rs`
2. Implement handlers in `src/handlers.rs`
3. Add routes in `src/main.rs`
4. Update database schema if needed in `migrations/`

## Troubleshooting

### Port Already in Use

If port 3000 is already in use, change it in `.env`:
```bash
PORT=8080
```

### Permission Denied on Upload Directory

Ensure the application has write permissions:
```bash
chmod 755 uploads
```

### Database Locked Error

Only one process can write to SQLite at a time. Ensure no other instances are running:
```bash
ps aux | grep fileshare_rust
```

### Tailscale Connection Issues

Check Tailscale status:
```bash
tailscale status
```

Restart Tailscale if needed:
```bash
sudo systemctl restart tailscaled
```

### Build Errors on Raspberry Pi

If you encounter memory issues during compilation:
```bash
# Increase swap space
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Change CONF_SWAPSIZE to 1024 or 2048
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

## Performance Considerations

- **File Size Limits**: Limited by available RAM (files are loaded into memory during upload)
- **Concurrent Uploads**: Handled by Tokio's async runtime
- **Database Connections**: Default pool size is 5 connections
- **Storage**: Files are stored on local filesystem, ensure adequate disk space

## Security Considerations

- **Authentication**: This basic version has no authentication. Add authentication middleware for production use.
- **File Validation**: Add file type and size validation as needed
- **CORS**: Configure appropriate CORS policies for production
- **HTTPS**: Use a reverse proxy (like nginx) with SSL/TLS for production
- **Rate Limiting**: Consider adding rate limiting for public deployments

## Future Enhancements

Potential improvements:
- User authentication and authorization
- File sharing with expiring links
- Image thumbnails and previews
- Search and filtering capabilities
- Chunked upload for large files
- Download progress tracking
- File versioning
- Compression support

## React Frontend Integration

For complete React frontend integration guide with code examples, see [API_DOCUMENTATION.md](./API_DOCUMENTATION.md).

Quick start:
```javascript
const API_BASE_URL = 'http://100.x.x.x:3000'; // Your Tailscale IP

// Upload file
const formData = new FormData();
formData.append('file', file);
fetch(`${API_BASE_URL}/api/files`, {
  method: 'POST',
  body: formData
});
```

## License

MIT License - feel free to use this project for personal or commercial purposes.

## Contributing

Feel free to submit issues and enhancement requests!

## Support

For questions or issues:
1. Check the troubleshooting section
2. Review the API documentation
3. Check application logs with `sudo journalctl -u fileshare.service`
