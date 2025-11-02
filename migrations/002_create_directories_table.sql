-- Create directories table
CREATE TABLE IF NOT EXISTS directories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES directories(id) ON DELETE CASCADE
);

-- Add parent_directory_id to files table
ALTER TABLE files ADD COLUMN parent_directory_id TEXT REFERENCES directories(id) ON DELETE CASCADE;

-- Create index on parent_directory_id for faster lookups
CREATE INDEX idx_files_parent_directory ON files(parent_directory_id);

-- Create index on directories parent_id for faster tree traversal
CREATE INDEX idx_directories_parent ON directories(parent_id);

-- Create index on directory name for search
CREATE INDEX idx_directories_name ON directories(name);
