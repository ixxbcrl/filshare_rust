# File Transfer API Documentation

## Base URL
```
http://localhost:3000
```

For Raspberry Pi deployment via Tailscale, replace `localhost:3000` with your Tailscale IP/hostname and port.

## API Endpoints

### 1. Health Check

Check if the API is running.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "service": "file-transfer-api"
}
```

**React Example:**
```javascript
const checkHealth = async () => {
  const response = await fetch('http://localhost:3000/health');
  const data = await response.json();
  console.log(data);
};
```

---

### 2. Upload File

Upload a file to the server with optional description.

**Endpoint:** `POST /api/files`

**Content-Type:** `multipart/form-data`

**Form Fields:**
- `file` (required): The file to upload
- `description` (optional): Text description of the file

**Response:**
```json
{
  "success": true,
  "file": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "550e8400-e29b-41d4-a716-446655440000.pdf",
    "original_filename": "document.pdf",
    "file_size": 1024000,
    "mime_type": "application/pdf",
    "uploaded_at": "2024-01-15T10:30:00Z",
    "description": "Important document"
  },
  "message": "File uploaded successfully"
}
```

**React Example:**
```javascript
const uploadFile = async (file, description = '') => {
  const formData = new FormData();
  formData.append('file', file);
  if (description) {
    formData.append('description', description);
  }

  try {
    const response = await fetch('http://localhost:3000/api/files', {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      throw new Error('Upload failed');
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error uploading file:', error);
    throw error;
  }
};

// Usage in component
const handleFileUpload = async (event) => {
  const file = event.target.files[0];
  const description = 'My file description';

  try {
    const result = await uploadFile(file, description);
    console.log('Uploaded:', result.file);
  } catch (error) {
    console.error('Upload error:', error);
  }
};
```

---

### 3. List All Files

Get a list of all uploaded files with metadata.

**Endpoint:** `GET /api/files`

**Response:**
```json
{
  "files": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "filename": "550e8400-e29b-41d4-a716-446655440000.pdf",
      "original_filename": "document.pdf",
      "file_size": 1024000,
      "mime_type": "application/pdf",
      "uploaded_at": "2024-01-15T10:30:00Z",
      "description": "Important document"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "filename": "660e8400-e29b-41d4-a716-446655440001.jpg",
      "original_filename": "photo.jpg",
      "file_size": 2048000,
      "mime_type": "image/jpeg",
      "uploaded_at": "2024-01-15T11:00:00Z",
      "description": null
    }
  ],
  "total": 2
}
```

**React Example:**
```javascript
const listFiles = async () => {
  try {
    const response = await fetch('http://localhost:3000/api/files');

    if (!response.ok) {
      throw new Error('Failed to fetch files');
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error fetching files:', error);
    throw error;
  }
};

// Usage in component with React hooks
import { useState, useEffect } from 'react';

const FileList = () => {
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchFiles = async () => {
      try {
        const data = await listFiles();
        setFiles(data.files);
      } catch (error) {
        console.error('Error:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchFiles();
  }, []);

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h2>Files ({files.length})</h2>
      <ul>
        {files.map(file => (
          <li key={file.id}>
            <strong>{file.original_filename}</strong>
            <br />
            Size: {(file.file_size / 1024).toFixed(2)} KB
            <br />
            Uploaded: {new Date(file.uploaded_at).toLocaleString()}
            {file.description && (
              <>
                <br />
                Description: {file.description}
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
};
```

---

### 4. Get File Information

Get metadata for a specific file.

**Endpoint:** `GET /api/files/:id`

**Path Parameters:**
- `id` (string): The UUID of the file

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "filename": "550e8400-e29b-41d4-a716-446655440000.pdf",
  "original_filename": "document.pdf",
  "file_size": 1024000,
  "mime_type": "application/pdf",
  "uploaded_at": "2024-01-15T10:30:00Z",
  "description": "Important document"
}
```

**Error Response (404):**
```json
{
  "error": "File not found"
}
```

**React Example:**
```javascript
const getFileInfo = async (fileId) => {
  try {
    const response = await fetch(`http://localhost:3000/api/files/${fileId}`);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('File not found');
      }
      throw new Error('Failed to fetch file info');
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error fetching file info:', error);
    throw error;
  }
};

// Usage in component
const FileInfo = ({ fileId }) => {
  const [fileInfo, setFileInfo] = useState(null);

  useEffect(() => {
    const fetchInfo = async () => {
      try {
        const info = await getFileInfo(fileId);
        setFileInfo(info);
      } catch (error) {
        console.error('Error:', error);
      }
    };

    fetchInfo();
  }, [fileId]);

  if (!fileInfo) return <div>Loading...</div>;

  return (
    <div>
      <h3>{fileInfo.original_filename}</h3>
      <p>Size: {(fileInfo.file_size / 1024 / 1024).toFixed(2)} MB</p>
      <p>Type: {fileInfo.mime_type || 'Unknown'}</p>
      <p>Uploaded: {new Date(fileInfo.uploaded_at).toLocaleString()}</p>
      {fileInfo.description && <p>Description: {fileInfo.description}</p>}
    </div>
  );
};
```

---

### 5. Download File

Download a file from the server.

**Endpoint:** `GET /api/files/:id/download`

**Path Parameters:**
- `id` (string): The UUID of the file

**Response:**
- Binary file stream
- Headers:
  - `Content-Type`: The MIME type of the file
  - `Content-Disposition`: `attachment; filename="original_filename"`

**Error Response (404):**
```json
{
  "error": "File not found"
}
```

**React Example:**
```javascript
const downloadFile = async (fileId, originalFilename) => {
  try {
    const response = await fetch(
      `http://localhost:3000/api/files/${fileId}/download`
    );

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('File not found');
      }
      throw new Error('Download failed');
    }

    // Create blob from response
    const blob = await response.blob();

    // Create download link
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = originalFilename;
    document.body.appendChild(link);
    link.click();

    // Cleanup
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  } catch (error) {
    console.error('Error downloading file:', error);
    throw error;
  }
};

// Usage in component
const DownloadButton = ({ fileId, filename }) => {
  const [downloading, setDownloading] = useState(false);

  const handleDownload = async () => {
    setDownloading(true);
    try {
      await downloadFile(fileId, filename);
    } catch (error) {
      alert('Download failed: ' + error.message);
    } finally {
      setDownloading(false);
    }
  };

  return (
    <button onClick={handleDownload} disabled={downloading}>
      {downloading ? 'Downloading...' : 'Download'}
    </button>
  );
};
```

---

### 6. Delete File

Delete a file from the server (both filesystem and database).

**Endpoint:** `DELETE /api/files/:id`

**Path Parameters:**
- `id` (string): The UUID of the file

**Response:**
```json
{
  "success": true,
  "message": "File deleted successfully"
}
```

**Error Response (404):**
```json
{
  "error": "File not found"
}
```

**React Example:**
```javascript
const deleteFile = async (fileId) => {
  try {
    const response = await fetch(
      `http://localhost:3000/api/files/${fileId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('File not found');
      }
      throw new Error('Delete failed');
    }

    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error deleting file:', error);
    throw error;
  }
};

// Usage in component
const DeleteButton = ({ fileId, onDeleted }) => {
  const [deleting, setDeleting] = useState(false);

  const handleDelete = async () => {
    if (!window.confirm('Are you sure you want to delete this file?')) {
      return;
    }

    setDeleting(true);
    try {
      await deleteFile(fileId);
      alert('File deleted successfully');
      onDeleted(fileId); // Callback to update UI
    } catch (error) {
      alert('Delete failed: ' + error.message);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <button onClick={handleDelete} disabled={deleting}>
      {deleting ? 'Deleting...' : 'Delete'}
    </button>
  );
};
```

---

## Complete React Example Application

Here's a complete example of a React component that uses all the API endpoints:

```javascript
import React, { useState, useEffect } from 'react';

const API_BASE_URL = 'http://localhost:3000';

const FileTransferApp = () => {
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(true);
  const [uploadFile, setUploadFile] = useState(null);
  const [description, setDescription] = useState('');
  const [uploading, setUploading] = useState(false);

  // Fetch files on component mount
  useEffect(() => {
    fetchFiles();
  }, []);

  const fetchFiles = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${API_BASE_URL}/api/files`);
      const data = await response.json();
      setFiles(data.files);
    } catch (error) {
      console.error('Error fetching files:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFileChange = (event) => {
    setUploadFile(event.target.files[0]);
  };

  const handleUpload = async (event) => {
    event.preventDefault();

    if (!uploadFile) {
      alert('Please select a file');
      return;
    }

    const formData = new FormData();
    formData.append('file', uploadFile);
    if (description) {
      formData.append('description', description);
    }

    try {
      setUploading(true);
      const response = await fetch(`${API_BASE_URL}/api/files`, {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) throw new Error('Upload failed');

      const data = await response.json();
      alert('File uploaded successfully!');

      // Reset form
      setUploadFile(null);
      setDescription('');
      event.target.reset();

      // Refresh file list
      fetchFiles();
    } catch (error) {
      alert('Upload error: ' + error.message);
    } finally {
      setUploading(false);
    }
  };

  const handleDownload = async (fileId, filename) => {
    try {
      const response = await fetch(
        `${API_BASE_URL}/api/files/${fileId}/download`
      );

      if (!response.ok) throw new Error('Download failed');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);
    } catch (error) {
      alert('Download error: ' + error.message);
    }
  };

  const handleDelete = async (fileId) => {
    if (!window.confirm('Are you sure you want to delete this file?')) {
      return;
    }

    try {
      const response = await fetch(`${API_BASE_URL}/api/files/${fileId}`, {
        method: 'DELETE',
      });

      if (!response.ok) throw new Error('Delete failed');

      alert('File deleted successfully!');
      fetchFiles();
    } catch (error) {
      alert('Delete error: ' + error.message);
    }
  };

  const formatFileSize = (bytes) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    return (bytes / 1024 / 1024).toFixed(2) + ' MB';
  };

  return (
    <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
      <h1>File Transfer Application</h1>

      {/* Upload Form */}
      <div style={{ marginBottom: '30px', padding: '20px', border: '1px solid #ccc' }}>
        <h2>Upload File</h2>
        <form onSubmit={handleUpload}>
          <div style={{ marginBottom: '10px' }}>
            <input
              type="file"
              onChange={handleFileChange}
              disabled={uploading}
            />
          </div>
          <div style={{ marginBottom: '10px' }}>
            <input
              type="text"
              placeholder="Description (optional)"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              disabled={uploading}
              style={{ width: '300px', padding: '5px' }}
            />
          </div>
          <button type="submit" disabled={uploading || !uploadFile}>
            {uploading ? 'Uploading...' : 'Upload'}
          </button>
        </form>
      </div>

      {/* File List */}
      <div>
        <h2>Files ({files.length})</h2>
        {loading ? (
          <p>Loading files...</p>
        ) : files.length === 0 ? (
          <p>No files uploaded yet.</p>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #333' }}>
                <th style={{ textAlign: 'left', padding: '10px' }}>Filename</th>
                <th style={{ textAlign: 'left', padding: '10px' }}>Size</th>
                <th style={{ textAlign: 'left', padding: '10px' }}>Type</th>
                <th style={{ textAlign: 'left', padding: '10px' }}>Uploaded</th>
                <th style={{ textAlign: 'left', padding: '10px' }}>Description</th>
                <th style={{ textAlign: 'left', padding: '10px' }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {files.map((file) => (
                <tr key={file.id} style={{ borderBottom: '1px solid #ddd' }}>
                  <td style={{ padding: '10px' }}>{file.original_filename}</td>
                  <td style={{ padding: '10px' }}>{formatFileSize(file.file_size)}</td>
                  <td style={{ padding: '10px' }}>{file.mime_type || 'Unknown'}</td>
                  <td style={{ padding: '10px' }}>
                    {new Date(file.uploaded_at).toLocaleString()}
                  </td>
                  <td style={{ padding: '10px' }}>{file.description || '-'}</td>
                  <td style={{ padding: '10px' }}>
                    <button
                      onClick={() => handleDownload(file.id, file.original_filename)}
                      style={{ marginRight: '5px' }}
                    >
                      Download
                    </button>
                    <button onClick={() => handleDelete(file.id)}>
                      Delete
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};

export default FileTransferApp;
```

---

## Error Handling

All endpoints return error responses in the following format:

```json
{
  "error": "Error message description"
}
```

Common HTTP status codes:
- `200 OK`: Success
- `400 Bad Request`: Invalid request data
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

---

## Data Types

### FileMetadata Object

```typescript
interface FileMetadata {
  id: string;                    // UUID
  filename: string;               // Stored filename with UUID
  original_filename: string;      // Original uploaded filename
  file_size: number;             // Size in bytes
  mime_type: string | null;      // MIME type (e.g., "image/jpeg")
  uploaded_at: string;           // ISO 8601 timestamp
  description: string | null;    // Optional description
}
```

---

## Configuration for Production

When deploying to Raspberry Pi with Tailscale:

1. Update the `API_BASE_URL` in your React app:
```javascript
const API_BASE_URL = 'http://your-tailscale-hostname:3000';
// or
const API_BASE_URL = 'http://100.x.x.x:3000'; // Tailscale IP
```

2. Ensure the Rust backend is configured to listen on `0.0.0.0` (it already is by default)

3. Make sure the port is accessible through your firewall if needed

---

## Notes

- Maximum file size is limited by available system memory
- Files are stored with UUID-based filenames to prevent conflicts
- CORS is enabled for all origins in development (adjust for production)
- All timestamps are in ISO 8601 format (UTC)
- File operations are atomic (database and filesystem stay in sync)
