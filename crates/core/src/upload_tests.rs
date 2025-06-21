//! Comprehensive tests for file upload module

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::super::upload::*;
    use wasm_bindgen_test::*;
    use web_sys::File;
    use js_sys::{Uint8Array, Array};
    
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper function to create a mock file
    fn create_mock_file(name: &str, content: &str, _mime_type: &str) -> File {
        let bytes = content.as_bytes();
        let array = Uint8Array::new_with_length(bytes.len() as u32);
        array.copy_from(bytes);
        
        let file_parts = Array::new();
        file_parts.push(&array.buffer());
        
        // Using simpler File constructor without options
        File::new_with_blob_sequence(
            &file_parts.into(),
            name,
        ).unwrap()
    }

    #[wasm_bindgen_test]
    fn test_upload_status_enum() {
        let pending = UploadStatus::Pending;
        let uploading = UploadStatus::Uploading;
        let complete = UploadStatus::Complete;
        let failed = UploadStatus::Failed("Error".to_string());
        
        assert_eq!(pending, UploadStatus::Pending);
        assert_eq!(uploading, UploadStatus::Uploading);
        assert_eq!(complete, UploadStatus::Complete);
        assert!(matches!(failed, UploadStatus::Failed(_)));
    }

    #[wasm_bindgen_test]
    fn test_file_upload_manager_creation() {
        let manager = FileUploadManager::new();
        assert_eq!(manager.max_file_size, 10 * 1024 * 1024); // 10MB
        assert_eq!(manager.allowed_types.len(), 4);
        assert!(manager.allowed_types.contains(&"image/jpeg".to_string()));
        assert!(manager.allowed_types.contains(&"image/png".to_string()));
        assert!(manager.allowed_types.contains(&"image/gif".to_string()));
        assert!(manager.allowed_types.contains(&"application/pdf".to_string()));
        assert!(manager.get_uploads().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_file_upload_manager_configuration() {
        let manager = FileUploadManager::new()
            .with_max_size(5 * 1024 * 1024) // 5MB
            .with_allowed_types(vec![
                "image/jpeg".to_string(),
                "text/plain".to_string(),
            ]);
        
        assert_eq!(manager.max_file_size, 5 * 1024 * 1024);
        assert_eq!(manager.allowed_types.len(), 2);
        assert!(manager.allowed_types.contains(&"image/jpeg".to_string()));
        assert!(manager.allowed_types.contains(&"text/plain".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_file_validation_size() {
        let manager = FileUploadManager::new()
            .with_max_size(1024); // 1KB
        
        // Create a small file (should pass)
        let small_file = create_mock_file("small.txt", "Hello", "text/plain");
        let result = manager.validate_file(&small_file);
        assert!(result.is_ok());
        
        // Create a large file (should fail)
        let large_content = "x".repeat(2000); // 2KB
        let large_file = create_mock_file("large.txt", &large_content, "text/plain");
        let result = manager.validate_file(&large_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File too large"));
    }

    #[wasm_bindgen_test]
    fn test_file_validation_type() {
        let manager = FileUploadManager::new()
            .with_allowed_types(vec!["text/plain".to_string()]);
        
        // Create allowed file type
        let text_file = create_mock_file("test.txt", "content", "text/plain");
        let result = manager.validate_file(&text_file);
        assert!(result.is_ok());
        
        // Create disallowed file type
        let image_file = create_mock_file("test.jpg", "content", "image/jpeg");
        let result = manager.validate_file(&image_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File type not allowed"));
    }

    #[wasm_bindgen_test]
    fn test_get_upload_by_file() {
        let mut manager = FileUploadManager::new();
        let file = create_mock_file("test.txt", "content", "text/plain");
        
        // Initially no upload
        assert!(manager.get_upload_by_file(&file).is_none());
        
        // Add an upload
        manager.uploads.push(FileUpload {
            file: file.clone(),
            progress: 50.0,
            status: UploadStatus::Uploading,
        });
        
        // Should find the upload
        let upload = manager.get_upload_by_file(&file);
        assert!(upload.is_some());
        assert_eq!(upload.unwrap().progress, 50.0);
    }

    #[wasm_bindgen_test]
    fn test_clear_completed() {
        let mut manager = FileUploadManager::new();
        
        // Add various uploads
        manager.uploads.push(FileUpload {
            file: create_mock_file("completed.txt", "done", "text/plain"),
            progress: 100.0,
            status: UploadStatus::Complete,
        });
        
        manager.uploads.push(FileUpload {
            file: create_mock_file("uploading.txt", "in progress", "text/plain"),
            progress: 50.0,
            status: UploadStatus::Uploading,
        });
        
        manager.uploads.push(FileUpload {
            file: create_mock_file("pending.txt", "waiting", "text/plain"),
            progress: 0.0,
            status: UploadStatus::Pending,
        });
        
        assert_eq!(manager.uploads.len(), 3);
        
        // Clear completed
        manager.clear_completed();
        assert_eq!(manager.uploads.len(), 2);
        
        // Verify only non-completed remain
        for upload in &manager.uploads {
            assert_ne!(upload.status, UploadStatus::Complete);
        }
    }

    #[wasm_bindgen_test]
    fn test_clear_failed() {
        let mut manager = FileUploadManager::new();
        
        // Add various uploads
        manager.uploads.push(FileUpload {
            file: create_mock_file("failed.txt", "error", "text/plain"),
            progress: 0.0,
            status: UploadStatus::Failed("Network error".to_string()),
        });
        
        manager.uploads.push(FileUpload {
            file: create_mock_file("uploading.txt", "in progress", "text/plain"),
            progress: 50.0,
            status: UploadStatus::Uploading,
        });
        
        assert_eq!(manager.uploads.len(), 2);
        
        // Clear failed
        manager.clear_failed();
        assert_eq!(manager.uploads.len(), 1);
        
        // Verify no failed uploads remain
        for upload in &manager.uploads {
            assert!(!matches!(upload.status, UploadStatus::Failed(_)));
        }
    }

    #[wasm_bindgen_test]
    fn test_retry_failed() {
        let mut manager = FileUploadManager::new();
        let file = create_mock_file("retry.txt", "content", "text/plain");
        
        // Add a failed upload
        manager.uploads.push(FileUpload {
            file: file.clone(),
            progress: 0.0,
            status: UploadStatus::Failed("Network error".to_string()),
        });
        
        // Retry the failed upload
        let upload = manager.retry_failed(&file);
        assert!(upload.is_some());
        
        let upload = upload.unwrap();
        assert_eq!(upload.status, UploadStatus::Pending);
        assert_eq!(upload.progress, 0.0);
    }

    #[wasm_bindgen_test]
    fn test_file_upload_component_creation() {
        let component = FileUploadComponent::new();
        assert_eq!(component.upload_url, "/api/upload");
        
        let component_with_url = FileUploadComponent::new()
            .with_url("/custom/upload".to_string());
        assert_eq!(component_with_url.upload_url, "/custom/upload");
    }

    #[wasm_bindgen_test]
    fn test_file_upload_component_configuration() {
        let component = FileUploadComponent::new()
            .with_max_size(1024 * 1024) // 1MB
            .with_allowed_types(vec!["image/png".to_string()]);
        
        assert_eq!(component.manager.max_file_size, 1024 * 1024);
        assert_eq!(component.manager.allowed_types.len(), 1);
        assert!(component.manager.allowed_types.contains(&"image/png".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_file_upload_component_status_rendering() {
        let mut component = FileUploadComponent::new();
        
        // Add some uploads with different statuses
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("pending.txt", "waiting", "text/plain"),
            progress: 0.0,
            status: UploadStatus::Pending,
        });
        
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("uploading.txt", "in progress", "text/plain"),
            progress: 45.0,
            status: UploadStatus::Uploading,
        });
        
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("complete.txt", "done", "text/plain"),
            progress: 100.0,
            status: UploadStatus::Complete,
        });
        
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("failed.txt", "error", "text/plain"),
            progress: 0.0,
            status: UploadStatus::Failed("Server error".to_string()),
        });
        
        let status_html = component.get_upload_status();
        
        // Check that all statuses are represented
        assert!(status_html.contains("pending.txt"));
        assert!(status_html.contains("Pending"));
        assert!(status_html.contains("uploading.txt"));
        assert!(status_html.contains("Uploading... 45%"));
        assert!(status_html.contains("complete.txt"));
        assert!(status_html.contains("Complete"));
        assert!(status_html.contains("failed.txt"));
        assert!(status_html.contains("Failed: Server error"));
    }

    #[wasm_bindgen_test]
    fn test_file_upload_component_render() {
        let component = FileUploadComponent::new()
            .with_max_size(5 * 1024 * 1024) // 5MB
            .with_allowed_types(vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "application/pdf".to_string(),
            ]);
        
        let html = component.render();
        
        // Check key elements are present
        assert!(html.contains("file-upload-component"));
        assert!(html.contains("upload-area"));
        assert!(html.contains("file-input"));
        assert!(html.contains("upload-label"));
        assert!(html.contains("Max size: 5 MB"));
        assert!(html.contains("image/jpeg, image/png, application/pdf"));
    }

    #[wasm_bindgen_test]
    fn test_file_upload_component_clear_methods() {
        let mut component = FileUploadComponent::new();
        
        // Add uploads with different statuses
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("complete.txt", "done", "text/plain"),
            progress: 100.0,
            status: UploadStatus::Complete,
        });
        
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("failed.txt", "error", "text/plain"),
            progress: 0.0,
            status: UploadStatus::Failed("Error".to_string()),
        });
        
        component.manager.uploads.push(FileUpload {
            file: create_mock_file("uploading.txt", "progress", "text/plain"),
            progress: 50.0,
            status: UploadStatus::Uploading,
        });
        
        assert_eq!(component.manager.uploads.len(), 3);
        
        // Clear completed
        component.clear_completed();
        assert_eq!(component.manager.uploads.len(), 2);
        
        // Clear failed
        component.clear_failed();
        assert_eq!(component.manager.uploads.len(), 1);
        
        // Only uploading should remain
        assert_eq!(component.manager.uploads[0].status, UploadStatus::Uploading);
    }
}