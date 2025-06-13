//! File upload support for Layer9

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{File, FormData, Response, Headers, Request, RequestInit};
use wasm_bindgen_futures::JsFuture;

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
    pub(crate) uploads: Vec<FileUpload>,
    pub(crate) max_file_size: u64,
    pub(crate) allowed_types: Vec<String>,
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
    
    pub async fn upload_file(&mut self, file: File, url: &str) -> Result<String, String> {
        // Validate file first
        self.validate_file(&file)?;
        
        // Create form data
        let form_data = FormData::new().map_err(|_| "Failed to create form data")?;
        form_data.append_with_blob("file", &file).map_err(|_| "Failed to append file")?;
        
        // Add file metadata
        form_data.append_with_str("filename", &file.name()).map_err(|_| "Failed to append filename")?;
        form_data.append_with_str("size", &file.size().to_string()).map_err(|_| "Failed to append size")?;
        form_data.append_with_str("type", &file.type_()).map_err(|_| "Failed to append type")?;
        
        // Create upload tracking
        let mut upload = FileUpload {
            file: file.clone(),
            progress: 0.0,
            status: UploadStatus::Uploading,
        };
        
        // Create request
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(web_sys::RequestMode::Cors);
        opts.set_body(&form_data.into());
        
        // Add auth header if we have a token
        let headers = Headers::new().map_err(|_| "Failed to create headers")?;
        if let Some(token) = crate::auth::JwtAuthProvider::get_stored_token() {
            headers.set("Authorization", &format!("Bearer {}", token))
                .map_err(|_| "Failed to set auth header")?;
        }
        opts.set_headers(&headers.into());
        
        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|_| "Failed to create request")?;
        
        let window = web_sys::window().ok_or("No window found")?;
        
        // Perform the upload
        let promise = window.fetch_with_request(&request);
        let resp = JsFuture::from(promise).await
            .map_err(|_| "Upload request failed")?;
        
        let response: Response = resp.dyn_into()
            .map_err(|_| "Failed to get response")?;
        
        if response.ok() {
            upload.status = UploadStatus::Complete;
            upload.progress = 100.0;
            
            // Get response body for upload ID
            let body_promise = response.text()
                .map_err(|_| "Failed to get response body")?;
            let body = JsFuture::from(body_promise).await
                .map_err(|_| "Failed to read response body")?;
            
            let upload_id = body.as_string().unwrap_or_else(|| {
                format!("upload-{}", js_sys::Date::now() as u64)
            });
            
            self.uploads.push(upload);
            Ok(upload_id)
        } else {
            upload.status = UploadStatus::Failed(format!("HTTP {}", response.status()));
            self.uploads.push(upload);
            Err(format!("Upload failed with status: {}", response.status()))
        }
    }
    
    pub fn get_uploads(&self) -> &Vec<FileUpload> {
        &self.uploads
    }
    
    pub fn get_upload_by_file(&self, file: &File) -> Option<&FileUpload> {
        self.uploads.iter().find(|u| u.file.name() == file.name())
    }
    
    pub fn clear_completed(&mut self) {
        self.uploads.retain(|u| u.status != UploadStatus::Complete);
    }
    
    pub fn clear_failed(&mut self) {
        self.uploads.retain(|u| !matches!(u.status, UploadStatus::Failed(_)));
    }
    
    pub fn retry_failed(&mut self, file: &File) -> Option<&mut FileUpload> {
        let upload = self.uploads.iter_mut().find(|u| {
            u.file.name() == file.name() && matches!(u.status, UploadStatus::Failed(_))
        })?;
        
        upload.status = UploadStatus::Pending;
        upload.progress = 0.0;
        Some(upload)
    }
    
    pub async fn upload_multiple(&mut self, files: Vec<File>, url: &str) -> Vec<Result<String, String>> {
        let mut results = Vec::new();
        
        for file in files {
            let result = self.upload_file(file, url).await;
            results.push(result);
        }
        
        results
    }
}

#[wasm_bindgen]
pub struct FileUploadComponent {
    pub(crate) manager: FileUploadManager,
    pub(crate) upload_url: String,
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
            upload_url: "/api/upload".to_string(),
        }
    }
    
    pub fn with_url(mut self, url: String) -> Self {
        self.upload_url = url;
        self
    }
    
    pub fn with_max_size(mut self, size: u64) -> Self {
        self.manager = self.manager.with_max_size(size);
        self
    }
    
    pub fn with_allowed_types(mut self, types: Vec<String>) -> Self {
        self.manager = self.manager.with_allowed_types(types);
        self
    }
    
    pub async fn handle_file_select(&mut self, files: web_sys::FileList) -> Result<Vec<String>, String> {
        let mut file_vec = Vec::new();
        
        for i in 0..files.length() {
            if let Some(file) = files.item(i) {
                file_vec.push(file);
            }
        }
        
        let results = self.manager.upload_multiple(file_vec, &self.upload_url).await;
        
        let mut upload_ids = Vec::new();
        for result in results {
            match result {
                Ok(id) => upload_ids.push(id),
                Err(e) => return Err(e),
            }
        }
        
        Ok(upload_ids)
    }
    
    pub fn get_upload_status(&self) -> String {
        let uploads = self.manager.get_uploads();
        let mut status_html = String::from("<div class='upload-status'>");
        
        for upload in uploads {
            let status_class = match &upload.status {
                UploadStatus::Pending => "pending",
                UploadStatus::Uploading => "uploading",
                UploadStatus::Complete => "complete",
                UploadStatus::Failed(_) => "failed",
            };
            
            let status_text = match &upload.status {
                UploadStatus::Pending => "Pending",
                UploadStatus::Uploading => &format!("Uploading... {}%", upload.progress as u32),
                UploadStatus::Complete => "Complete",
                UploadStatus::Failed(e) => &format!("Failed: {}", e),
            };
            
            status_html.push_str(&format!(
                r#"<div class='upload-item {}'><span class='file-name'>{}</span><span class='status'>{}</span></div>"#,
                status_class,
                upload.file.name(),
                status_text
            ));
        }
        
        status_html.push_str("</div>");
        status_html
    }
    
    pub fn render(&self) -> String {
        format!(r#"<div class="file-upload-component">
            <div class="upload-area">
                <input type="file" id="file-input" multiple class="file-input" />
                <label for="file-input" class="upload-label">
                    <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
                    </svg>
                    <span>Click to select files or drag and drop</span>
                    <span class="file-types">Allowed: {}</span>
                    <span class="max-size">Max size: {} MB</span>
                </label>
            </div>
            <div id="upload-progress">{}</div>
        </div>"#,
            self.manager.allowed_types.join(", "),
            self.manager.max_file_size / (1024 * 1024),
            self.get_upload_status()
        )
    }
    
    pub fn clear_completed(&mut self) {
        self.manager.clear_completed();
    }
    
    pub fn clear_failed(&mut self) {
        self.manager.clear_failed();
    }
}
