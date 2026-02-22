use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileMetadata {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub uploaded_at: String,
    pub description: Option<String>,
    pub parent_directory_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Directory {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub uploaded_at: String,
    pub description: Option<String>,
    pub parent_directory_id: Option<String>,
}

impl From<FileMetadata> for FileResponse {
    fn from(metadata: FileMetadata) -> Self {
        Self {
            id: metadata.id,
            filename: metadata.filename,
            original_filename: metadata.original_filename,
            file_size: metadata.file_size,
            mime_type: metadata.mime_type,
            uploaded_at: metadata.uploaded_at,
            description: metadata.description,
            parent_directory_id: metadata.parent_directory_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DirectoryResponse {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_count: i64,
    pub total_size: i64,
}

impl From<Directory> for DirectoryResponse {
    fn from(directory: Directory) -> Self {
        Self {
            id: directory.id,
            name: directory.name,
            parent_id: directory.parent_id,
            created_at: directory.created_at,
            updated_at: directory.updated_at,
            file_count: 0,
            total_size: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub file: FileResponse,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ListFilesResponse {
    pub files: Vec<FileResponse>,
    pub directories: Vec<DirectoryResponse>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateDirectoryRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateDirectoryResponse {
    pub success: bool,
    pub directory: DirectoryResponse,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteRequest {
    pub file_ids: Vec<String>,
    pub directory_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveFileRequest {
    pub parent_directory_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveDirectoryRequest {
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkDeleteResponse {
    pub success: bool,
    pub deleted_files: usize,
    pub deleted_directories: usize,
    pub message: String,
}
