use crate::db::DbPool;
use crate::models::{Directory, FileMetadata};
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
        parent_directory_id: Option<String>,
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
            parent_directory_id: parent_directory_id.clone(),
        };

        sqlx::query(
            r#"
            INSERT INTO files (id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description, parent_directory_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(&metadata.parent_directory_id)
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
            "SELECT id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description, parent_directory_id FROM files WHERE id = ?"
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(metadata)
    }

    pub async fn list_files(&self, parent_directory_id: Option<String>) -> Result<Vec<FileMetadata>, sqlx::Error> {
        let files = if let Some(dir_id) = parent_directory_id {
            sqlx::query_as::<_, FileMetadata>(
                "SELECT id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description, parent_directory_id FROM files WHERE parent_directory_id = ? ORDER BY uploaded_at DESC"
            )
            .bind(dir_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, FileMetadata>(
                "SELECT id, filename, original_filename, file_size, mime_type, storage_path, uploaded_at, description, parent_directory_id FROM files WHERE parent_directory_id IS NULL ORDER BY uploaded_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

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

    // Directory management methods
    pub async fn create_directory(
        &self,
        name: &str,
        parent_id: Option<String>,
    ) -> Result<Directory, Box<dyn std::error::Error + Send + Sync>> {
        let dir_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let directory = Directory {
            id: dir_id.clone(),
            name: name.to_string(),
            parent_id: parent_id.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        sqlx::query(
            r#"
            INSERT INTO directories (id, name, parent_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&directory.id)
        .bind(&directory.name)
        .bind(&directory.parent_id)
        .bind(&directory.created_at)
        .bind(&directory.updated_at)
        .execute(&self.pool)
        .await?;

        info!("Directory created: {} ({})", name, dir_id);
        Ok(directory)
    }

    pub async fn list_directories(&self, parent_id: Option<String>) -> Result<Vec<Directory>, sqlx::Error> {
        let directories = if let Some(p_id) = parent_id {
            sqlx::query_as::<_, Directory>(
                "SELECT id, name, parent_id, created_at, updated_at FROM directories WHERE parent_id = ? ORDER BY name ASC"
            )
            .bind(p_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Directory>(
                "SELECT id, name, parent_id, created_at, updated_at FROM directories WHERE parent_id IS NULL ORDER BY name ASC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(directories)
    }

    pub async fn get_directory(&self, dir_id: &str) -> Result<Option<Directory>, sqlx::Error> {
        let directory = sqlx::query_as::<_, Directory>(
            "SELECT id, name, parent_id, created_at, updated_at FROM directories WHERE id = ?"
        )
        .bind(dir_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(directory)
    }

    pub async fn delete_directory(
        &self,
        dir_id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Check if directory exists
        let directory = self.get_directory(dir_id).await?;
        if directory.is_none() {
            return Ok(false);
        }

        // Delete all files in this directory
        sqlx::query("DELETE FROM files WHERE parent_directory_id = ?")
            .bind(dir_id)
            .execute(&self.pool)
            .await?;

        // Delete the directory (CASCADE will handle subdirectories and their files)
        let result = sqlx::query("DELETE FROM directories WHERE id = ?")
            .bind(dir_id)
            .execute(&self.pool)
            .await?;

        info!("Directory deleted: {}", dir_id);
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_directory_stats(&self, dir_id: &str) -> Result<(i64, i64), sqlx::Error> {
        // Get file count and total size for a directory
        let result: Option<(Option<i64>, Option<i64>)> = sqlx::query_as(
            "SELECT COUNT(*), SUM(file_size) FROM files WHERE parent_directory_id = ?"
        )
        .bind(dir_id)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((Some(count), Some(size))) => Ok((count, size)),
            Some((Some(count), None)) => Ok((count, 0)),
            _ => Ok((0, 0)),
        }
    }

    pub async fn move_file(
        &self,
        file_id: &str,
        parent_directory_id: Option<String>,
    ) -> Result<Option<FileMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "UPDATE files SET parent_directory_id = ? WHERE id = ?"
        )
        .bind(&parent_directory_id)
        .bind(file_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        let metadata = self.get_file_metadata(file_id).await?;
        info!("File moved: {} -> {:?}", file_id, parent_directory_id);
        Ok(metadata)
    }

    pub async fn move_directory(
        &self,
        dir_id: &str,
        parent_id: Option<String>,
    ) -> Result<Option<Directory>, Box<dyn std::error::Error + Send + Sync>> {
        // Prevent moving a directory into itself or one of its descendants
        if let Some(ref target_id) = parent_id {
            if target_id == dir_id {
                return Err("Cannot move a directory into itself".into());
            }
            if self.is_ancestor_of(dir_id, target_id).await? {
                return Err("Cannot move a directory into one of its own subdirectories".into());
            }
        }

        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE directories SET parent_id = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&parent_id)
        .bind(&now)
        .bind(dir_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        let directory = self.get_directory(dir_id).await?;
        info!("Directory moved: {} -> {:?}", dir_id, parent_id);
        Ok(directory)
    }

    /// Returns true if `ancestor_id` is an ancestor of `target_id` (walks up the tree).
    async fn is_ancestor_of(&self, ancestor_id: &str, target_id: &str) -> Result<bool, sqlx::Error> {
        let mut current = target_id.to_string();
        loop {
            match self.get_directory(&current).await? {
                None => return Ok(false),
                Some(dir) => match dir.parent_id {
                    None => return Ok(false),
                    Some(pid) => {
                        if pid == ancestor_id {
                            return Ok(true);
                        }
                        current = pid;
                    }
                },
            }
        }
    }

    pub async fn bulk_delete(
        &self,
        file_ids: Vec<String>,
        directory_ids: Vec<String>,
    ) -> Result<(usize, usize), Box<dyn std::error::Error + Send + Sync>> {
        let mut deleted_files = 0;
        let mut deleted_directories = 0;

        // Delete files
        for file_id in file_ids {
            if self.delete_file(&file_id).await? {
                deleted_files += 1;
            }
        }

        // Delete directories
        for dir_id in directory_ids {
            if self.delete_directory(&dir_id).await? {
                deleted_directories += 1;
            }
        }

        Ok((deleted_files, deleted_directories))
    }
}
