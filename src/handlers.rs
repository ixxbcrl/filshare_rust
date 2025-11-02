use crate::models::{
    BulkDeleteRequest, BulkDeleteResponse, CreateDirectoryRequest, CreateDirectoryResponse,
    DeleteResponse, DirectoryResponse, ErrorResponse, FileResponse, ListFilesResponse,
    UploadResponse,
};
use crate::storage::FileStorage;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub parent_directory_id: Option<String>,
}

// Upload file handler
pub async fn upload_file(
    State(storage): State<FileStorage>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut filename = String::new();
    let mut file_data = Vec::new();
    let mut mime_type: Option<String> = None;
    let mut description: Option<String> = None;
    let mut parent_directory_id: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            error!("Failed to read multipart field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to read multipart data: {}", e),
                }),
            )
        })?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                filename = field
                    .file_name()
                    .unwrap_or("unnamed")
                    .to_string();

                mime_type = field.content_type().map(|s| s.to_string());

                file_data = field.bytes().await.map_err(|e| {
                    error!("Failed to read file bytes: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read file data: {}", e),
                        }),
                    )
                })?.to_vec();
            }
            "description" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read description: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read description: {}", e),
                        }),
                    )
                })?;
                description = Some(text);
            }
            "parent_directory_id" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read parent_directory_id: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read parent_directory_id: {}", e),
                        }),
                    )
                })?;
                if !text.is_empty() {
                    parent_directory_id = Some(text);
                }
            }
            _ => {}
        }
    }

    if filename.is_empty() || file_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No file provided".to_string(),
            }),
        ));
    }

    let metadata = storage
        .save_file(&filename, &file_data, mime_type, description, parent_directory_id)
        .await
        .map_err(|e| {
            error!("Failed to save file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to save file: {}", e),
                }),
            )
        })?;

    info!("File uploaded successfully: {}", metadata.id);

    Ok(Json(UploadResponse {
        success: true,
        file: metadata.into(),
        message: "File uploaded successfully".to_string(),
    }))
}

// Download file handler
pub async fn download_file(
    State(storage): State<FileStorage>,
    Path(file_id): Path<String>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let metadata = storage
        .get_file_metadata(&file_id)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let metadata = metadata.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "File not found".to_string(),
            }),
        )
    })?;

    let file_path = storage
        .get_file_path(&file_id)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "File not found".to_string(),
                }),
            )
        })?;

    let file = File::open(&file_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to open file: {}", e),
            }),
        )
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_type = metadata
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", metadata.original_filename),
        )
        .body(body)
        .unwrap())
}

// List all files and directories handler
pub async fn list_files(
    State(storage): State<FileStorage>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListFilesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let files = storage
        .list_files(query.parent_directory_id.clone())
        .await
        .map_err(|e| {
            error!("Failed to list files: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to list files: {}", e),
                }),
            )
        })?;

    let directories = storage
        .list_directories(query.parent_directory_id)
        .await
        .map_err(|e| {
            error!("Failed to list directories: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to list directories: {}", e),
                }),
            )
        })?;

    // Get stats for each directory
    let mut directory_responses = Vec::new();
    for dir in directories {
        let (file_count, total_size) = storage.get_directory_stats(&dir.id).await.map_err(|e| {
            error!("Failed to get directory stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get directory stats: {}", e),
                }),
            )
        })?;

        directory_responses.push(DirectoryResponse {
            id: dir.id,
            name: dir.name,
            parent_id: dir.parent_id,
            created_at: dir.created_at,
            updated_at: dir.updated_at,
            file_count,
            total_size,
        });
    }

    let total = files.len() + directory_responses.len();
    let file_responses: Vec<FileResponse> = files.into_iter().map(|f| f.into()).collect();

    Ok(Json(ListFilesResponse {
        files: file_responses,
        directories: directory_responses,
        total,
    }))
}

// Get file metadata handler
pub async fn get_file_info(
    State(storage): State<FileStorage>,
    Path(file_id): Path<String>,
) -> Result<Json<FileResponse>, (StatusCode, Json<ErrorResponse>)> {
    let metadata = storage
        .get_file_metadata(&file_id)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?;

    let metadata = metadata.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "File not found".to_string(),
            }),
        )
    })?;

    Ok(Json(metadata.into()))
}

// Delete file handler
pub async fn delete_file(
    State(storage): State<FileStorage>,
    Path(file_id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let deleted = storage.delete_file(&file_id).await.map_err(|e| {
        error!("Failed to delete file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete file: {}", e),
            }),
        )
    })?;

    if deleted {
        info!("File deleted: {}", file_id);
        Ok(Json(DeleteResponse {
            success: true,
            message: "File deleted successfully".to_string(),
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "File not found".to_string(),
            }),
        ))
    }
}

// Health check handler
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "file-transfer-api"
    }))
}

// Create directory handler
pub async fn create_directory(
    State(storage): State<FileStorage>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> Result<Json<CreateDirectoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let directory = storage
        .create_directory(&payload.name, payload.parent_id)
        .await
        .map_err(|e| {
            error!("Failed to create directory: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create directory: {}", e),
                }),
            )
        })?;

    let (file_count, total_size) = storage.get_directory_stats(&directory.id).await.map_err(|e| {
        error!("Failed to get directory stats: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get directory stats: {}", e),
            }),
        )
    })?;

    info!("Directory created: {}", directory.id);

    Ok(Json(CreateDirectoryResponse {
        success: true,
        directory: DirectoryResponse {
            id: directory.id,
            name: directory.name,
            parent_id: directory.parent_id,
            created_at: directory.created_at,
            updated_at: directory.updated_at,
            file_count,
            total_size,
        },
        message: "Directory created successfully".to_string(),
    }))
}

// Get directory info handler
pub async fn get_directory_info(
    State(storage): State<FileStorage>,
    Path(dir_id): Path<String>,
) -> Result<Json<DirectoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let directory = storage
        .get_directory(&dir_id)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Directory not found".to_string(),
                }),
            )
        })?;

    let (file_count, total_size) = storage.get_directory_stats(&dir_id).await.map_err(|e| {
        error!("Failed to get directory stats: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get directory stats: {}", e),
            }),
        )
    })?;

    Ok(Json(DirectoryResponse {
        id: directory.id,
        name: directory.name,
        parent_id: directory.parent_id,
        created_at: directory.created_at,
        updated_at: directory.updated_at,
        file_count,
        total_size,
    }))
}

// Delete directory handler
pub async fn delete_directory(
    State(storage): State<FileStorage>,
    Path(dir_id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let deleted = storage.delete_directory(&dir_id).await.map_err(|e| {
        error!("Failed to delete directory: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete directory: {}", e),
            }),
        )
    })?;

    if deleted {
        info!("Directory deleted: {}", dir_id);
        Ok(Json(DeleteResponse {
            success: true,
            message: "Directory deleted successfully".to_string(),
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Directory not found".to_string(),
            }),
        ))
    }
}

// Bulk delete handler
pub async fn bulk_delete(
    State(storage): State<FileStorage>,
    Json(payload): Json<BulkDeleteRequest>,
) -> Result<Json<BulkDeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (deleted_files, deleted_directories) = storage
        .bulk_delete(payload.file_ids, payload.directory_ids)
        .await
        .map_err(|e| {
            error!("Failed to bulk delete: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to bulk delete: {}", e),
                }),
            )
        })?;

    info!(
        "Bulk delete completed: {} files, {} directories",
        deleted_files, deleted_directories
    );

    Ok(Json(BulkDeleteResponse {
        success: true,
        deleted_files,
        deleted_directories,
        message: format!(
            "Deleted {} files and {} directories",
            deleted_files, deleted_directories
        ),
    }))
}
