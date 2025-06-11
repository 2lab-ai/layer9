//! File Upload Support - L4/L5

use crate::component::Component;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{File, FormData};

/// File upload state
#[derive(Clone)]
pub struct UploadState {
    pub files: Vec<FileInfo>,
    pub uploading: bool,
    pub progress: f32,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub preview_url: Option<String>,
    pub upload_progress: f32,
    pub uploaded: bool,
    pub error: Option<String>,
}

/// File uploader
#[derive(Clone)]
pub struct FileUploader {
    state: Rc<RefCell<UploadState>>,
    config: UploadConfig,
}

#[derive(Clone)]
pub struct UploadConfig {
    pub url: String,
    pub max_file_size: u64,
    pub allowed_types: Vec<String>,
    pub multiple: bool,
    pub auto_upload: bool,
    pub on_progress: Option<Rc<dyn Fn(f32)>>,
    pub on_complete: Option<Rc<dyn Fn(Vec<UploadResult>)>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub file_name: String,
    pub url: String,
    pub size: u64,
}

impl FileUploader {
    pub fn new(config: UploadConfig) -> Self {
        FileUploader {
            state: Rc::new(RefCell::new(UploadState {
                files: vec![],
                uploading: false,
                progress: 0.0,
                errors: vec![],
            })),
            config,
        }
    }

    pub fn add_files(&self, files: Vec<File>) {
        let mut state = self.state.borrow_mut();
        state.errors.clear();

        for file in files {
            // Validate file
            let size = file.size() as u64;
            let mime_type = file.type_();
            let name = file.name();

            // Check file size
            if size > self.config.max_file_size {
                state.errors.push(format!(
                    "File {} is too large. Maximum size is {} MB",
                    name,
                    self.config.max_file_size / 1_000_000
                ));
                continue;
            }

            // Check file type
            if !self.config.allowed_types.is_empty()
                && !self.config.allowed_types.contains(&mime_type)
            {
                state
                    .errors
                    .push(format!("File type {} is not allowed", mime_type));
                continue;
            }

            // Create preview for images
            let preview_url = if mime_type.starts_with("image/") {
                Some(create_object_url(&file))
            } else {
                None
            };

            state.files.push(FileInfo {
                name,
                size,
                mime_type,
                preview_url,
                upload_progress: 0.0,
                uploaded: false,
                error: None,
            });
        }

        // Auto upload if enabled
        if self.config.auto_upload && !state.files.is_empty() {
            drop(state);
            self.upload_all();
        }
    }

    pub fn upload_all(&self) {
        let state = self.state.clone();
        let config_url = self.config.url.clone();

        spawn_local(async move {
            state.borrow_mut().uploading = true;

            let files_count = state.borrow().files.len();
            let mut uploaded_files = vec![];

            for (index, _file_info) in state.borrow().files.clone().iter().enumerate() {
                // Create form data
                let form_data = FormData::new().unwrap();

                // Add file (in real implementation, we'd need the actual File object)
                // form_data.append_with_blob("file", &file);

                // Upload file
                match upload_file(&config_url, form_data).await {
                    Ok(result) => {
                        state.borrow_mut().files[index].uploaded = true;
                        state.borrow_mut().files[index].upload_progress = 100.0;
                        uploaded_files.push(result);
                    }
                    Err(error) => {
                        state.borrow_mut().files[index].error = Some(error);
                    }
                }

                // Update overall progress
                let progress = ((index + 1) as f32 / files_count as f32) * 100.0;
                state.borrow_mut().progress = progress;
            }

            state.borrow_mut().uploading = false;

            // Call completion handler
            // if let Some(on_complete) = &config.on_complete {
            //     on_complete(uploaded_files);
            // }
        });
    }

    pub fn remove_file(&self, index: usize) {
        let mut state = self.state.borrow_mut();
        if index < state.files.len() {
            // Revoke object URL if it exists
            if let Some(url) = &state.files[index].preview_url {
                revoke_object_url(url);
            }
            state.files.remove(index);
        }
    }

    pub fn clear(&self) {
        let mut state = self.state.borrow_mut();
        // Revoke all object URLs
        for file in &state.files {
            if let Some(url) = &file.preview_url {
                revoke_object_url(url);
            }
        }
        state.files.clear();
        state.errors.clear();
        state.progress = 0.0;
    }
}

/// Upload file to server
async fn upload_file(url: &str, form_data: FormData) -> Result<UploadResult, String> {
    let response = fetch_with_form_data(url, form_data)
        .await
        .map_err(|e| format!("Upload failed: {:?}", e))?;

    if response.ok() {
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {:?}", e))
    } else {
        Err(format!("Upload failed with status: {}", response.status()))
    }
}

/// Fetch with FormData
async fn fetch_with_form_data(url: &str, form_data: FormData) -> Result<FetchResponse, FetchError> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form_data.into());

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let response_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;

    let response: web_sys::Response = response_value.dyn_into()?;
    Ok(FetchResponse::from_web_sys(response))
}

/// Create object URL for preview
fn create_object_url(file: &File) -> String {
    web_sys::Url::create_object_url_with_blob(file).unwrap_or_default()
}

/// Revoke object URL
fn revoke_object_url(url: &str) {
    let _ = web_sys::Url::revoke_object_url(url);
}

/// File upload component
pub struct FileUploadComponent {
    uploader: FileUploader,
    accept: Option<String>,
    label: String,
}

impl FileUploadComponent {
    pub fn new(uploader: FileUploader) -> Self {
        FileUploadComponent {
            uploader,
            accept: None,
            label: "Choose files".to_string(),
        }
    }

    pub fn accept(mut self, accept: impl Into<String>) -> Self {
        self.accept = Some(accept.into());
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Component for FileUploadComponent {
    fn render(&self) -> Element {
        let state = self.uploader.state.borrow();
        let uploader = self.uploader.clone();

        let errors_element = if !state.errors.is_empty() {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("upload-errors".to_string()),
                    ..Default::default()
                },
                children: state
                    .errors
                    .iter()
                    .map(|error| Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("error".to_string()),
                            ..Default::default()
                        },
                        children: vec![Element::Text(error.clone())],
                    })
                    .collect(),
            }
        } else {
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![],
            }
        };

        let files_element = if !state.files.is_empty() {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("file-list".to_string()),
                    ..Default::default()
                },
                children: state
                    .files
                    .iter()
                    .enumerate()
                    .map(|(index, file)| {
                        let preview_element = if let Some(preview) = &file.preview_url {
                            Element::Node {
                                tag: "img".to_string(),
                                props: Props {
                                    class: Some("file-preview".to_string()),
                                    attributes: vec![("src".to_string(), preview.clone())],
                                    ..Default::default()
                                },
                                children: vec![],
                            }
                        } else {
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props {
                                    class: Some("file-icon".to_string()),
                                    ..Default::default()
                                },
                                children: vec![],
                            }
                        };

                        let status_element = if !file.uploaded
                            && file.upload_progress > 0.0
                            && file.upload_progress < 100.0
                        {
                            Progress::new(file.upload_progress).render()
                        } else if file.uploaded {
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props {
                                    class: Some("upload-success".to_string()),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("✓".to_string())],
                            }
                        } else if let Some(error) = &file.error {
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props {
                                    class: Some("upload-error".to_string()),
                                    ..Default::default()
                                },
                                children: vec![Element::Text(error.clone())],
                            }
                        } else {
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props::default(),
                                children: vec![],
                            }
                        };

                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("file-item".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                preview_element,
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("file-info".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "div".to_string(),
                                            props: Props {
                                                class: Some("file-name".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text(file.name.clone())],
                                        },
                                        Element::Node {
                                            tag: "div".to_string(),
                                            props: Props {
                                                class: Some("file-size".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text(format_file_size(
                                                file.size,
                                            ))],
                                        },
                                    ],
                                },
                                status_element,
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        on_click: Some(Rc::new(move || {
                                            uploader.remove_file(index)
                                        })),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("×".to_string())],
                                },
                            ],
                        }
                    })
                    .collect(),
            }
        } else {
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![],
            }
        };

        let progress_element = if state.uploading {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("upload-progress".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Progress::new(state.progress).render(),
                    Element::Node {
                        tag: "span".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(format!(
                            "Uploading... {}%",
                            state.progress.round()
                        ))],
                    },
                ],
            }
        } else if !state.files.is_empty() && !self.uploader.config.auto_upload {
            Element::Node {
                tag: "button".to_string(),
                props: Props {
                    on_click: Some(Rc::new(move || uploader.upload_all())),
                    ..Default::default()
                },
                children: vec![Element::Text("Upload All".to_string())],
            }
        } else {
            Element::Node {
                tag: "div".to_string(),
                props: Props::default(),
                children: vec![],
            }
        };

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("file-upload".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "input".to_string(),
                    props: Props {
                        id: Some("file-input".to_string()),
                        attributes: vec![
                            ("type".to_string(), "file".to_string()),
                            (
                                "accept".to_string(),
                                self.accept.as_deref().unwrap_or("*").to_string(),
                            ),
                            (
                                "multiple".to_string(),
                                self.uploader.config.multiple.to_string(),
                            ),
                            (
                                "onchange".to_string(),
                                "handleFileSelect(event)".to_string(),
                            ),
                            ("style".to_string(), "display: none".to_string()),
                        ],
                        ..Default::default()
                    },
                    children: vec![],
                },
                Element::Node {
                    tag: "label".to_string(),
                    props: Props {
                        class: Some("upload-button".to_string()),
                        attributes: vec![("for".to_string(), "file-input".to_string())],
                        ..Default::default()
                    },
                    children: vec![Element::Text(self.label.clone())],
                },
                errors_element,
                files_element,
                progress_element,
            ],
        }
    }
}

/// Format file size for display
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Hook for file uploads
pub fn use_file_upload(config: UploadConfig) -> FileUploader {
    FileUploader::new(config)
}

/// Alias for use_file_upload
pub fn use_upload(config: UploadConfig) -> FileUploader {
    use_file_upload(config)
}

// Re-exports
use crate::fetch::{FetchError, FetchResponse};
use crate::ui::Progress;
use wasm_bindgen_futures::spawn_local;
