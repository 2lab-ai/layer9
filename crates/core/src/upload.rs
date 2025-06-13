//! File upload support for Layer9

use wasm_bindgen::prelude::*;
use web_sys::{File, FormData};

#[derive(Debug, Clone)]
pub struct FileUpload {
    pub file: File,
    pub progress: f64,
    pub status: UploadStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Complete,
    Failed(String),
}

pub struct FileUploadManager {
    uploads: Vec<FileUpload>,
    max_file_size: u64,
    allowed_types: Vec<String>,
}

impl Default for FileUploadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileUploadManager {
    pub fn new() -> Self {
        Self {
            uploads: Vec::new(),
            max_file_size: 10 * 1024 * 1024, // 10MB default
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "application/pdf".to_string(),
            ],
        }
    }
    
    pub fn with_max_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }
    
    pub fn with_allowed_types(mut self, types: Vec<String>) -> Self {
        self.allowed_types = types;
        self
    }
    
    pub fn validate_file(&self, file: &File) -> Result<(), String> {
        // Check file size
        let size = file.size() as u64;
        if size > self.max_file_size {
            return Err(format!("File too large. Maximum size: {} bytes", self.max_file_size));
        }
        
        // Check file type
        let file_type = file.type_();
        if !self.allowed_types.contains(&file_type) {
            return Err(format!("File type not allowed: {}", file_type));
        }
        
        Ok(())
    }
    
    pub async fn upload_file(&mut self, file: File, _url: &str) -> Result<String, String> {
        // Validate file first
        self.validate_file(&file)?;
        
        // Create form data
        let form_data = FormData::new().map_err(|_| "Failed to create form data")?;
        form_data.append_with_blob("file", &file).map_err(|_| "Failed to append file")?;
        
        // In a real implementation, this would use fetch API to upload
        // For now, we'll simulate the upload
        let upload = FileUpload {
            file,
            progress: 0.0,
            status: UploadStatus::Uploading,
        };
        
        self.uploads.push(upload);
        
        // Simulate successful upload
        Ok("upload-id-123".to_string())
    }
    
    pub fn get_uploads(&self) -> &Vec<FileUpload> {
        &self.uploads
    }
}

#[wasm_bindgen]
pub struct FileUploadComponent {
    #[allow(dead_code)]
    manager: FileUploadManager,
}

impl Default for FileUploadComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl FileUploadComponent {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            manager: FileUploadManager::new(),
        }
    }
    
    pub fn render(&self) -> String {
        r#"<div class="file-upload">
            <input type="file" id="file-input" multiple />
            <button onclick="uploadFiles()">Upload</button>
            <div id="upload-progress"></div>
        </div>"#.to_string()
    }
}
