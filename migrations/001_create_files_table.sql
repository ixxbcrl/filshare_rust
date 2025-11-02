-- Create files table
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type TEXT,
    storage_path TEXT NOT NULL,
    uploaded_at TEXT NOT NULL,
    description TEXT
);

-- Create index on uploaded_at for faster sorting
CREATE INDEX idx_files_uploaded_at ON files(uploaded_at DESC);
