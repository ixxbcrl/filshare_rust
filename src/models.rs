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
    pub total: usize,
}
