use layer9_core::prelude::*;
use layer9_core::form::{use_form, Form, FormConfig, FormFields};
use layer9_core::component::{Element, Props};
use layer9_core::reactive_v2::mount;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Event;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

impl FormFields for LoginForm {
    fn get_field(&self, field: &str) -> Option<String> {
        match field {
            "username" => Some(self.username.clone()),
            "password" => Some(self.password.clone()),
            _ => None,
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> Result<(), String> {
        match field {
            "username" => {
                self.username = value;
                Ok(())
            }
            "password" => {
                self.password = value;
                Ok(())
            }
            _ => Err(format!("Unknown field: {}", field)),
        }
    }
    
    fn field_names(&self) -> Vec<&'static str> {
        vec!["username", "password"]
    }
}

struct LoginComponent {
    form: Form<LoginForm>,
}

impl LoginComponent {
    fn new() -> Self {
        let form = use_form(FormConfig {
            initial_values: LoginForm::default(),
            validate: Some(Box::new(|values| {
                let mut errors = HashMap::new();
                
                // Validate username
                if values.username.trim().is_empty() {
                    errors.entry("username".to_string())
                        .or_insert_with(Vec::new)
                        .push("Username is required".to_string());
                } else if values.username.len() < 3 {
                    errors.entry("username".to_string())
                        .or_insert_with(Vec::new)
                        .push("Username must be at least 3 characters".to_string());
                }
                
                // Validate password
                if values.password.trim().is_empty() {
                    errors.entry("password".to_string())
                        .or_insert_with(Vec::new)
                        .push("Password is required".to_string());
                } else if values.password.len() < 6 {
                    errors.entry("password".to_string())
                        .or_insert_with(Vec::new)
                        .push("Password must be at least 6 characters".to_string());
                }
                
                errors
            })),
            on_submit: Box::new(|values| {
                let values_clone = values.clone();
                Box::pin(async move {
                    web_sys::console::log_1(&format!("Login attempt: {:?}", values_clone).into());
                    // Simulate API call
                    Ok(())
                })
            }),
        });
        
        LoginComponent { form }
    }
}

impl Component for LoginComponent {
    fn render(&self) -> Element {
        let values = self.form.values();
        let errors = self.form.errors();
        let is_submitting = self.form.is_submitting();
        
        let form_clone = self.form.clone();
        let username_handler = move |value: String| {
            form_clone.set_field_value("username", value);
        };
        
        let form_clone = self.form.clone();
        let password_handler = move |value: String| {
            form_clone.set_field_value("password", value);
        };
        
        let form_clone = self.form.clone();
        let submit_handler = move |_event: Event| {
            form_clone.handle_submit()();
        };
        
        let username_error = if let Some(errs) = errors.get("username") {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("error".to_string()),
                    ..Default::default()
                },
                children: vec![Element::Text(errs.join(", "))],
            }
        } else {
            Element::Text("".to_string())
        };
        
        let password_error = if let Some(errs) = errors.get("password") {
            Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("error".to_string()),
                    ..Default::default()
                },
                children: vec![Element::Text(errs.join(", "))],
            }
        } else {
            Element::Text("".to_string())
        };
        
        let button_text = if is_submitting {
            "Logging in..."
        } else {
            "Login"
        };
        
        // Create input elements with proper attributes
        let username_input = Element::Node {
            tag: "input".to_string(),
            props: Props {
                attributes: vec![
                    ("type".to_string(), "text".to_string()),
                    ("value".to_string(), values.username.clone()),
                    ("placeholder".to_string(), "Enter username".to_string()),
                ],
                on_change: Some(Rc::new(username_handler)),
                ..Default::default()
            },
            children: vec![],
        };
        
        let password_input = Element::Node {
            tag: "input".to_string(),
            props: Props {
                attributes: vec![
                    ("type".to_string(), "password".to_string()),
                    ("value".to_string(), values.password.clone()),
                    ("placeholder".to_string(), "Enter password".to_string()),
                ],
                on_change: Some(Rc::new(password_handler)),
                ..Default::default()
            },
            children: vec![],
        };
        
        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("container".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "h1".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Login Form Demo".to_string())],
                },
                Element::Node {
                    tag: "form".to_string(),
                    props: Props {
                        on_submit: Some(Rc::new(submit_handler)),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("form-group".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "label".to_string(),
                                    props: Props::default(),
                                    children: vec![Element::Text("Username".to_string())],
                                },
                                username_input,
                                username_error,
                            ],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("form-group".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "label".to_string(),
                                    props: Props::default(),
                                    children: vec![Element::Text("Password".to_string())],
                                },
                                password_input,
                                password_error,
                            ],
                        },
                        Element::Node {
                            tag: "button".to_string(),
                            props: Props::default(),
                            children: vec![Element::Text(button_text.to_string())],
                        },
                    ],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("debug".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "h3".to_string(),
                            props: Props::default(),
                            children: vec![Element::Text("Form State Debug".to_string())],
                        },
                        Element::Node {
                            tag: "pre".to_string(),
                            props: Props::default(),
                            children: vec![Element::Text(format!("{:#?}", values))],
                        },
                    ],
                },
            ],
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Create and mount the app
    mount(Box::new(LoginComponent::new()), "root");
    
    // Add some basic styles
    let style = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("style")
        .unwrap();
    
    style.set_text_content(Some(r#"
        .container {
            max-width: 400px;
            margin: 2rem auto;
            padding: 2rem;
            font-family: system-ui, -apple-system, sans-serif;
        }
        
        h1 {
            margin-bottom: 2rem;
            color: #333;
        }
        
        .form-group {
            margin-bottom: 1.5rem;
        }
        
        label {
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 500;
            color: #555;
        }
        
        input {
            width: 100%;
            padding: 0.5rem;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 1rem;
            box-sizing: border-box;
        }
        
        input:focus {
            outline: none;
            border-color: #4CAF50;
        }
        
        .error {
            color: #d32f2f;
            font-size: 0.875rem;
            margin-top: 0.25rem;
        }
        
        button {
            width: 100%;
            padding: 0.75rem;
            background-color: #4CAF50;
            color: white;
            border: none;
            border-radius: 4px;
            font-size: 1rem;
            cursor: pointer;
            transition: background-color 0.2s;
        }
        
        button:hover {
            background-color: #45a049;
        }
        
        button:disabled {
            background-color: #ccc;
            cursor: not-allowed;
        }
        
        .debug {
            margin-top: 2rem;
            padding: 1rem;
            background-color: #f5f5f5;
            border-radius: 4px;
        }
        
        .debug h3 {
            margin-top: 0;
            color: #666;
        }
        
        pre {
            margin: 0;
            font-size: 0.875rem;
            color: #333;
        }
    "#));
    
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .head()
        .unwrap()
        .append_child(&style)
        .unwrap();
}