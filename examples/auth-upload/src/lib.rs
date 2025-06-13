//! Layer9 Authentication and File Upload Demo
//! 
//! This example demonstrates:
//! - User authentication with JWT
//! - Protected file upload functionality
//! - Session management
//! - Role-based access control

use layer9_core::{
    auth::{AuthService, JwtAuthProvider, AuthContext},
    upload::FileUploadComponent,
};
use wasm_bindgen::prelude::*;
use web_sys::{console, FileList};

#[wasm_bindgen]
pub struct AuthUploadApp {
    auth_service: AuthService,
    upload_component: FileUploadComponent,
    state_version: u32,
}

#[wasm_bindgen]
impl AuthUploadApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        
        // Initialize JWT auth provider with demo users
        let mut jwt_provider = JwtAuthProvider::new("layer9-demo-secret".to_string());
        
        // Add demo users with different roles
        jwt_provider.add_user(
            "admin".to_string(),
            "admin123".to_string(),
            "admin@layer9.dev".to_string(),
            vec!["admin".to_string()],
        );
        
        jwt_provider.add_user(
            "user".to_string(),
            "user123".to_string(),
            "user@layer9.dev".to_string(),
            vec!["user".to_string()],
        );
        
        jwt_provider.add_user(
            "guest".to_string(),
            "guest123".to_string(),
            "guest@layer9.dev".to_string(),
            vec!["guest".to_string()],
        );
        
        let auth_service = AuthService::new(Box::new(jwt_provider));
        
        // Configure file upload component
        let upload_component = FileUploadComponent::new()
            .with_url("/api/upload".to_string())
            .with_max_size(5 * 1024 * 1024) // 5MB
            .with_allowed_types(vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "application/pdf".to_string(),
                "text/plain".to_string(),
            ]);
        
        Self {
            auth_service,
            upload_component,
            state_version: 0,
        }
    }
    
    pub fn render(&self) -> String {
        let auth_context = self.auth_service.get_context();
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Layer9 Auth & Upload Demo</title>
    <style>{}</style>
</head>
<body>
    <div class="container">
        <h1>🔐 Layer9 Authentication & File Upload Demo</h1>
        
        {}
        
        <div class="demo-info">
            <h3>Demo Users:</h3>
            <ul>
                <li><strong>admin/admin123</strong> - Full access (can upload)</li>
                <li><strong>user/user123</strong> - User access (can upload)</li>
                <li><strong>guest/guest123</strong> - Guest access (no upload)</li>
            </ul>
        </div>
    </div>
    
    <script>{}</script>
</body>
</html>
        "#, 
            self.get_styles(),
            self.render_main_content(&auth_context),
            self.get_scripts()
        )
    }
    
    fn render_main_content(&self, auth_context: &AuthContext) -> String {
        if auth_context.is_authenticated() {
            self.render_authenticated_view(auth_context)
        } else {
            self.render_login_form()
        }
    }
    
    fn render_authenticated_view(&self, auth_context: &AuthContext) -> String {
        let user = auth_context.user.as_ref().unwrap();
        let can_upload = auth_context.has_permission("write");
        
        format!(r#"
        <div class="auth-section">
            <h2>Welcome, {}!</h2>
            <div class="user-info">
                <p><strong>Email:</strong> {}</p>
                <p><strong>Roles:</strong> {}</p>
                <p><strong>Permissions:</strong> {}</p>
            </div>
            <button onclick="window.app.handle_logout()" class="btn btn-secondary">Logout</button>
        </div>
        
        <div class="upload-section">
            <h2>📤 File Upload</h2>
            {}
        </div>
        "#,
            user.username,
            user.email,
            user.roles.join(", "),
            auth_context.permissions.join(", "),
            if can_upload {
                self.upload_component.render()
            } else {
                r#"<div class="no-permission">⚠️ You don't have permission to upload files. Login as 'admin' or 'user' to upload.</div>"#.to_string()
            }
        )
    }
    
    fn render_login_form(&self) -> String {
        r#"
        <div class="login-section">
            <h2>Login</h2>
            <form id="login-form" onsubmit="return false;">
                <div class="form-group">
                    <label>Username:</label>
                    <input type="text" id="username" placeholder="Enter username" required>
                </div>
                <div class="form-group">
                    <label>Password:</label>
                    <input type="password" id="password" placeholder="Enter password" required>
                </div>
                <button type="submit" onclick="window.app.handle_login()" class="btn btn-primary">Login</button>
                <div id="error-message" class="error-message"></div>
            </form>
        </div>
        "#.to_string()
    }
    
    fn get_styles(&self) -> &str {
        r#"
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #333;
        }
        
        .container {
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            padding: 40px;
            max-width: 800px;
            width: 90%;
        }
        
        h1 {
            color: #333;
            margin-bottom: 30px;
            text-align: center;
        }
        
        h2 {
            color: #555;
            margin-bottom: 20px;
        }
        
        .auth-section, .login-section, .upload-section {
            margin-bottom: 30px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }
        
        .user-info {
            margin: 20px 0;
            padding: 15px;
            background: white;
            border-radius: 6px;
        }
        
        .user-info p {
            margin: 8px 0;
        }
        
        .form-group {
            margin-bottom: 20px;
        }
        
        label {
            display: block;
            margin-bottom: 8px;
            font-weight: 500;
            color: #666;
        }
        
        input[type="text"],
        input[type="password"] {
            width: 100%;
            padding: 12px;
            border: 2px solid #e1e4e8;
            border-radius: 6px;
            font-size: 16px;
            transition: border-color 0.3s;
        }
        
        input[type="text"]:focus,
        input[type="password"]:focus {
            outline: none;
            border-color: #667eea;
        }
        
        .btn {
            padding: 12px 24px;
            border: none;
            border-radius: 6px;
            font-size: 16px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.3s;
        }
        
        .btn-primary {
            background: #667eea;
            color: white;
        }
        
        .btn-primary:hover {
            background: #5a67d8;
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }
        
        .btn-secondary {
            background: #e1e4e8;
            color: #333;
        }
        
        .btn-secondary:hover {
            background: #d1d5db;
        }
        
        .error-message {
            color: #e53e3e;
            margin-top: 15px;
            padding: 10px;
            background: #fff5f5;
            border-radius: 6px;
            display: none;
        }
        
        .error-message.show {
            display: block;
        }
        
        .demo-info {
            background: #e6f3ff;
            border: 1px solid #b3d9ff;
            border-radius: 8px;
            padding: 20px;
            margin-top: 30px;
        }
        
        .demo-info h3 {
            color: #0066cc;
            margin-bottom: 15px;
        }
        
        .demo-info ul {
            list-style: none;
            padding-left: 0;
        }
        
        .demo-info li {
            margin: 10px 0;
            padding: 8px;
            background: white;
            border-radius: 4px;
        }
        
        .no-permission {
            padding: 20px;
            background: #fff5f5;
            border: 1px solid #feb2b2;
            color: #c53030;
            border-radius: 6px;
            text-align: center;
        }
        
        /* File upload styles */
        .file-upload-component {
            margin-top: 20px;
        }
        
        .upload-area {
            border: 2px dashed #cbd5e0;
            border-radius: 8px;
            padding: 40px;
            text-align: center;
            transition: all 0.3s;
        }
        
        .upload-area:hover {
            border-color: #667eea;
            background: #f7fafc;
        }
        
        .file-input {
            display: none;
        }
        
        .upload-label {
            cursor: pointer;
            display: inline-block;
        }
        
        .upload-label svg {
            stroke: #667eea;
            margin-bottom: 10px;
        }
        
        .file-types, .max-size {
            display: block;
            margin-top: 10px;
            font-size: 14px;
            color: #718096;
        }
        
        .upload-status {
            margin-top: 20px;
        }
        
        .upload-item {
            padding: 12px;
            margin: 8px 0;
            border-radius: 6px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .upload-item.pending {
            background: #e6f3ff;
            color: #0066cc;
        }
        
        .upload-item.uploading {
            background: #fff7e6;
            color: #cc6600;
        }
        
        .upload-item.complete {
            background: #e6ffe6;
            color: #008800;
        }
        
        .upload-item.failed {
            background: #ffe6e6;
            color: #cc0000;
        }
        "#
    }
    
    fn get_scripts(&self) -> &str {
        r#"
        // Store app instance globally
        window.app = {
            handle_login: async function() {
                const username = document.getElementById('username').value;
                const password = document.getElementById('password').value;
                const errorMsg = document.getElementById('error-message');
                
                try {
                    const response = await window.wasmApp.login(username, password);
                    if (response.success) {
                        location.reload();
                    } else {
                        errorMsg.textContent = response.error;
                        errorMsg.classList.add('show');
                    }
                } catch (e) {
                    errorMsg.textContent = 'Login failed: ' + e.message;
                    errorMsg.classList.add('show');
                }
            },
            
            handle_logout: function() {
                window.wasmApp.logout();
                location.reload();
            },
            
            handle_file_select: async function(event) {
                const files = event.target.files;
                if (files && files.length > 0) {
                    try {
                        await window.wasmApp.upload_files(files);
                        // Update UI to show upload progress
                        window.updateUploadStatus();
                    } catch (e) {
                        console.error('Upload failed:', e);
                    }
                }
            },
            
            updateUploadStatus: function() {
                // This would be called periodically to update upload progress
                const statusHtml = window.wasmApp.get_upload_status();
                const statusContainer = document.getElementById('upload-progress');
                if (statusContainer) {
                    statusContainer.innerHTML = statusHtml;
                }
            }
        };
        
        // Set up file input handler when DOM is ready
        document.addEventListener('DOMContentLoaded', function() {
            const fileInput = document.getElementById('file-input');
            if (fileInput) {
                fileInput.addEventListener('change', window.app.handle_file_select);
            }
        });
        "#
    }
    
    #[wasm_bindgen]
    pub async fn login(&mut self, username: String, password: String) -> JsValue {
        match self.auth_service.login(&username, &password).await {
            Ok(_) => {
                let result = js_sys::Object::new();
                js_sys::Reflect::set(&result, &"success".into(), &true.into()).unwrap();
                result.into()
            }
            Err(e) => {
                let result = js_sys::Object::new();
                js_sys::Reflect::set(&result, &"success".into(), &false.into()).unwrap();
                js_sys::Reflect::set(&result, &"error".into(), &e.into()).unwrap();
                result.into()
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn logout(&mut self) {
        self.auth_service.logout();
        self.state_version += 1;
    }
    
    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.auth_service.is_authenticated()
    }
    
    #[wasm_bindgen]
    pub async fn upload_files(&mut self, files: FileList) -> Result<JsValue, JsValue> {
        let auth_context = self.auth_service.get_context();
        
        // Check if user has upload permission
        if !auth_context.has_permission("write") {
            return Err(JsValue::from_str("You don't have permission to upload files"));
        }
        
        // Use the file upload component
        match self.upload_component.handle_file_select(files).await {
            Ok(upload_ids) => {
                let array = js_sys::Array::new();
                for id in upload_ids {
                    array.push(&JsValue::from_str(&id));
                }
                Ok(array.into())
            }
            Err(e) => Err(JsValue::from_str(&e))
        }
    }
    
    #[wasm_bindgen]
    pub fn get_upload_status(&self) -> String {
        self.upload_component.get_upload_status()
    }
}

// Initialize the app when WASM loads
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"Layer9 Auth & Upload Demo initialized".into());
}