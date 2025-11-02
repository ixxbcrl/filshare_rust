use crate::db::DbPool;
use crate::models::FileMetadata;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct FileStorage {
    upload_dir: PathBuf,
    pool: DbPool,
}

impl FileStorage {
    pub fn new(upload_dir: PathBuf, pool: DbPool) -> Self {
        Self { upload_dir, pool }
    }

    pub async fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.upload_dir).await?;
        info!("Upload directory initialized at: {:?}", self.upload_dir);
        Ok(())
    }

    pub async fn save_file(
        &self,
        filename: &str,
        content: &[u8],
        mime_type: Option<String>,
        description: Option<String>,
    ) -> Result<FileMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let file_id = Uuid::new_v4().to_string();
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let stored_filename = if extension.is_empty() {
            file_id.clone()
        } else {
            format!("{}.{}", file_id, extension)
        };

        let file_path = self.upload_dir.join(&stored_filename);

        // Write file to disk
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(content).await?;
        file.flush().await?;

        let file_size = content.len() as i64;
        let uploaded_at = Utc::now().to_rfc3339();

        // Save metadata to database
        let metadata = FileMetadata {
            id: file_id.clone(),
            filename: stored_filename.clone(),
            original_filename: filename.to_string(),
            file_size,
            mime_type: mime_type.clone(),
            storage_path: file_path.to_string_lossy().to_string(),
            uploaded_at: uploaded_at.clone(),
            description: description.clone(),
        };

        sqlx::query(
            r#"
            INSERT INTO files (id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&metadata.id)
        .bind(&metadata.filename)
        .bind(&metadata.original_filename)
        .bind(metadata.file_size)
        .bind(&metadata.mime_type)
        .bind(&metadata.storage_path)
        .bind(&metadata.uploaded_at)
        .bind(&metadata.description)
        .execute(&self.pool)
        .await?;

        info!("File saved: {} ({})", filename, file_id);
        Ok(metadata)
    }

    pub async fn get_file_metadata(
        &self,
        file_id: &str,
    ) -> Result<Option<FileMetadata>, sqlx::Error> {
        let metadata = sqlx::query_as::<_, FileMetadata>(
            "SELECT id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description FROM files WHERE id = ?"
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(metadata)
    }

    pub async fn list_files(&self) -> Result<Vec<FileMetadata>, sqlx::Error> {
        let files = sqlx::query_as::<_, FileMetadata>(
            "SELECT id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description FROM files ORDER BY uploaded_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(files)
    }

    pub async fn delete_file(
        &self,
        file_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Get file metadata first
        let metadata = self.get_file_metadata(file_id).await?;

        if let Some(meta) = metadata {
            // Delete from filesystem
            let file_path = Path::new(&meta.storage_path);
            if file_path.exists() {
                fs::remove_file(file_path).await?;
                info!("File deleted from filesystem: {:?}", file_path);
            }

            // Delete from database
            let result = sqlx::query("DELETE FROM files WHERE id = ?")
                .bind(file_id)
                .execute(&self.pool)
                .await?;

            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }

    pub async fn get_file_path(&self, file_id: &str) -> Result<Option<PathBuf>, sqlx::Error> {
        let metadata = self.get_file_metadata(file_id).await?;
        Ok(metadata.map(|m| PathBuf::from(m.storage_path)))
    }
}
