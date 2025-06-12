//! Beautiful Form Validation Example
//! Showcases real-time validation, error handling, and stunning UI

use layer9_core::prelude::*;
use layer9_core::hooks::use_state;
use layer9_core::reactive_v2::mount;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Default)]
struct FormData {
    name: String,
    email: String,
    password: String,
    confirm_password: String,
    age: String,
    terms: bool,
}

#[derive(Clone, Default)]
struct FormErrors {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    confirm_password: Option<String>,
    age: Option<String>,
    terms: Option<String>,
}

struct BeautifulForm;

impl Component for BeautifulForm {
    fn render(&self) -> Element {
        let (form_data, set_form_data) = use_state(FormData::default());
        let (errors, set_errors) = use_state(FormErrors::default());
        let (submitted, set_submitted) = use_state(false);
        let (submitting, set_submitting) = use_state(false);
        
        // Validation functions
        let validate_name = |name: &str| -> Option<String> {
            if name.is_empty() {
                Some("Name is required".to_string())
            } else if name.len() < 2 {
                Some("Name must be at least 2 characters".to_string())
            } else if name.len() > 50 {
                Some("Name must be less than 50 characters".to_string())
            } else {
                None
            }
        };
        
        let validate_email = |email: &str| -> Option<String> {
            if email.is_empty() {
                Some("Email is required".to_string())
            } else if !email.contains('@') || !email.contains('.') {
                Some("Please enter a valid email address".to_string())
            } else {
                None
            }
        };
        
        let validate_password = |password: &str| -> Option<String> {
            if password.is_empty() {
                Some("Password is required".to_string())
            } else if password.len() < 8 {
                Some("Password must be at least 8 characters".to_string())
            } else if !password.chars().any(|c| c.is_uppercase()) {
                Some("Password must contain at least one uppercase letter".to_string())
            } else if !password.chars().any(|c| c.is_numeric()) {
                Some("Password must contain at least one number".to_string())
            } else {
                None
            }
        };
        
        let validate_confirm_password = |password: &str, confirm: &str| -> Option<String> {
            if confirm.is_empty() {
                Some("Please confirm your password".to_string())
            } else if password != confirm {
                Some("Passwords do not match".to_string())
            } else {
                None
            }
        };
        
        let validate_age = |age: &str| -> Option<String> {
            if age.is_empty() {
                Some("Age is required".to_string())
            } else if let Ok(age_num) = age.parse::<i32>() {
                if age_num < 18 {
                    Some("You must be at least 18 years old".to_string())
                } else if age_num > 120 {
                    Some("Please enter a valid age".to_string())
                } else {
                    None
                }
            } else {
                Some("Please enter a valid number".to_string())
            }
        };
        
        let validate_form = {
            let form_data = form_data.clone();
            move || -> FormErrors {
                FormErrors {
                    name: validate_name(&form_data.name),
                    email: validate_email(&form_data.email),
                    password: validate_password(&form_data.password),
                    confirm_password: validate_confirm_password(&form_data.password, &form_data.confirm_password),
                    age: validate_age(&form_data.age),
                    terms: if !form_data.terms { Some("You must accept the terms".to_string()) } else { None },
                }
            }
        };
        
        let handle_submit = {
            let form_data = form_data.clone();
            let set_errors = set_errors.clone();
            let set_submitted = set_submitted.clone();
            let set_submitting = set_submitting.clone();
            let validate_form = validate_form.clone();
            move || {
                let validation_errors = validate_form();
                set_errors(validation_errors.clone());
                
                let has_errors = validation_errors.name.is_some() ||
                    validation_errors.email.is_some() ||
                    validation_errors.password.is_some() ||
                    validation_errors.confirm_password.is_some() ||
                    validation_errors.age.is_some() ||
                    validation_errors.terms.is_some();
                
                if !has_errors {
                    set_submitting(true);
                    // Simulate API call
                    let window = web_sys::window().unwrap();
                    let set_submitting = set_submitting.clone();
                    let set_submitted = set_submitted.clone();
                    let closure = Closure::wrap(Box::new(move || {
                        set_submitting(false);
                        set_submitted(true);
                    }) as Box<dyn FnMut()>);
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1500
                    ).unwrap();
                    closure.forget();
                }
            }
        };
        
        // Input handlers
        let update_field = |field: &str, value: String| {
            let mut data = form_data.clone();
            let mut errs = errors.clone();
            
            match field {
                "name" => {
                    data.name = value.clone();
                    errs.name = validate_name(&value);
                },
                "email" => {
                    data.email = value.clone();
                    errs.email = validate_email(&value);
                },
                "password" => {
                    data.password = value.clone();
                    errs.password = validate_password(&value);
                    if !data.confirm_password.is_empty() {
                        errs.confirm_password = validate_confirm_password(&value, &data.confirm_password);
                    }
                },
                "confirm_password" => {
                    data.confirm_password = value.clone();
                    errs.confirm_password = validate_confirm_password(&data.password, &value);
                },
                "age" => {
                    data.age = value.clone();
                    errs.age = validate_age(&value);
                },
                "terms" => {
                    data.terms = !data.terms;
                    errs.terms = if !data.terms { Some("You must accept the terms".to_string()) } else { None };
                },
                _ => {}
            }
            
            set_form_data(data);
            set_errors(errs);
        };

        if submitted {
            // Success state
            return Element::Node {
                tag: "div".to_string(),
                props: Props {
                    class: Some("form-container".to_string()),
                    ..Default::default()
                },
                children: vec![
                    Element::Node {
                        tag: "style".to_string(),
                        props: Props::default(),
                        children: vec![Element::Text(FORM_STYLES.to_string())],
                    },
                    Element::Node {
                        tag: "div".to_string(),
                        props: Props {
                            class: Some("success-card".to_string()),
                            ..Default::default()
                        },
                        children: vec![
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props {
                                    class: Some("success-icon".to_string()),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("✓".to_string())],
                            },
                            Element::Node {
                                tag: "h2".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text("Registration Successful!".to_string())],
                            },
                            Element::Node {
                                tag: "p".to_string(),
                                props: Props::default(),
                                children: vec![Element::Text(format!("Welcome, {}! Your account has been created.", form_data.name))],
                            },
                            Element::Node {
                                tag: "button".to_string(),
                                props: Props {
                                    class: Some("btn btn-primary".to_string()),
                                    on_click: Some(Rc::new(move || {
                                        web_sys::window().unwrap().location().reload().unwrap();
                                    })),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("Create Another Account".to_string())],
                            },
                        ],
                    },
                ],
            };
        }

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("form-container".to_string()),
                ..Default::default()
            },
            children: vec![
                // Inline styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(FORM_STYLES.to_string())],
                },
                
                // Background decoration
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("bg-decoration".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("shape shape-1".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("shape shape-2".to_string()),
                                ..Default::default()
                            },
                            children: vec![],
                        },
                    ],
                },
                
                // Form card
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("form-card".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        // Header
                        Element::Node {
                            tag: "header".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Node {
                                    tag: "h1".to_string(),
                                    props: Props::default(),
                                    children: vec![
                                        Element::Text("Create ".to_string()),
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("gradient-text".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("Account".to_string())],
                                        },
                                    ],
                                },
                                Element::Node {
                                    tag: "p".to_string(),
                                    props: Props {
                                        class: Some("subtitle".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Experience real-time validation with Layer9".to_string())],
                                },
                            ],
                        },
                        
                        // Form
                        Element::Node {
                            tag: "form".to_string(),
                            props: Props::default(),
                            children: vec![
                                // Name field
                                create_input_field(
                                    "Name",
                                    "text",
                                    "name",
                                    &form_data.name,
                                    errors.name.as_ref(),
                                    "Enter your full name",
                                    update_field.clone()
                                ),
                                
                                // Email field
                                create_input_field(
                                    "Email",
                                    "email",
                                    "email",
                                    &form_data.email,
                                    errors.email.as_ref(),
                                    "your@email.com",
                                    update_field.clone()
                                ),
                                
                                // Password field
                                create_input_field(
                                    "Password",
                                    "password",
                                    "password",
                                    &form_data.password,
                                    errors.password.as_ref(),
                                    "At least 8 characters",
                                    update_field.clone()
                                ),
                                
                                // Confirm password field
                                create_input_field(
                                    "Confirm Password",
                                    "password",
                                    "confirm_password",
                                    &form_data.confirm_password,
                                    errors.confirm_password.as_ref(),
                                    "Re-enter your password",
                                    update_field.clone()
                                ),
                                
                                // Age field
                                create_input_field(
                                    "Age",
                                    "number",
                                    "age",
                                    &form_data.age,
                                    errors.age.as_ref(),
                                    "18",
                                    update_field.clone()
                                ),
                                
                                // Terms checkbox
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("checkbox-group".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "label".to_string(),
                                            props: Props {
                                                class: Some("checkbox-label".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![
                                                Element::Node {
                                                    tag: "input".to_string(),
                                                    props: Props {
                                                        attributes: vec![
                                                            ("type".to_string(), "checkbox".to_string()),
                                                            if form_data.terms { ("checked".to_string(), "checked".to_string()) } else { ("".to_string(), "".to_string()) },
                                                        ].into_iter().filter(|(k, _)| !k.is_empty()).collect(),
                                                        on_click: Some(Rc::new({
                                                            let update_field = update_field.clone();
                                                            move || update_field("terms", String::new())
                                                        })),
                                                        ..Default::default()
                                                    },
                                                    children: vec![],
                                                },
                                                Element::Text(" I accept the ".to_string()),
                                                Element::Node {
                                                    tag: "a".to_string(),
                                                    props: Props {
                                                        attributes: vec![("href".to_string(), "#".to_string())],
                                                        ..Default::default()
                                                    },
                                                    children: vec![Element::Text("terms and conditions".to_string())],
                                                },
                                            ],
                                        },
                                        if let Some(error) = &errors.terms {
                                            Element::Node {
                                                tag: "span".to_string(),
                                                props: Props {
                                                    class: Some("error-message".to_string()),
                                                    ..Default::default()
                                                },
                                                children: vec![Element::Text(error.clone())],
                                            }
                                        } else {
                                            Element::Node {
                                                tag: "span".to_string(),
                                                props: Props::default(),
                                                children: vec![],
                                            }
                                        },
                                    ],
                                },
                                
                                // Submit button
                                Element::Node {
                                    tag: "button".to_string(),
                                    props: Props {
                                        class: Some(if submitting { "btn btn-primary loading" } else { "btn btn-primary" }.to_string()),
                                        attributes: vec![("type".to_string(), "button".to_string())],
                                        on_click: if submitting { None } else { Some(Rc::new(handle_submit)) },
                                        ..Default::default()
                                    },
                                    children: vec![
                                        if submitting {
                                            Element::Node {
                                                tag: "span".to_string(),
                                                props: Props {
                                                    class: Some("spinner".to_string()),
                                                    ..Default::default()
                                                },
                                                children: vec![],
                                            }
                                        } else {
                                            Element::Text("Create Account".to_string())
                                        },
                                    ],
                                },
                            ],
                        },
                        
                        // Footer
                        Element::Node {
                            tag: "footer".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Text("Built with ".to_string()),
                                Element::Node {
                                    tag: "a".to_string(),
                                    props: Props {
                                        attributes: vec![
                                            ("href".to_string(), "https://github.com/anthropics/layer9".to_string()),
                                            ("target".to_string(), "_blank".to_string()),
                                        ],
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Layer9".to_string())],
                                },
                                Element::Text(" • Form Validation Demo".to_string()),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}

fn create_input_field(
    label: &str,
    input_type: &str,
    field_name: &str,
    value: &str,
    error: Option<&String>,
    placeholder: &str,
    update_field: impl Fn(&str, String) + 'static,
) -> Element {
    let field_name = field_name.to_string();
    let has_error = error.is_some();
    
    Element::Node {
        tag: "div".to_string(),
        props: Props {
            class: Some(if has_error { "form-group error" } else { "form-group" }.to_string()),
            ..Default::default()
        },
        children: vec![
            Element::Node {
                tag: "label".to_string(),
                props: Props::default(),
                children: vec![Element::Text(label.to_string())],
            },
            Element::Node {
                tag: "input".to_string(),
                props: Props {
                    id: Some(field_name.clone()),
                    attributes: vec![
                        ("type".to_string(), input_type.to_string()),
                        ("placeholder".to_string(), placeholder.to_string()),
                        ("value".to_string(), value.to_string()),
                    ],
                    on_click: Some(Rc::new({
                        let field_name = field_name.clone();
                        move || {
                            // Get input value on change
                            let window = web_sys::window().unwrap();
                            let document = window.document().unwrap();
                            if let Some(input) = document.get_element_by_id(&field_name) {
                                if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                                    update_field(&field_name, input.value());
                                }
                            }
                        }
                    })),
                    ..Default::default()
                },
                children: vec![],
            },
            if let Some(err) = error {
                Element::Node {
                    tag: "span".to_string(),
                    props: Props {
                        class: Some("error-message".to_string()),
                        ..Default::default()
                    },
                    children: vec![Element::Text(err.clone())],
                }
            } else {
                Element::Node {
                    tag: "span".to_string(),
                    props: Props {
                        class: Some("helper-text".to_string()),
                        ..Default::default()
                    },
                    children: vec![],
                }
            },
        ],
    }
}

const FORM_STYLES: &str = r#"
    :root {
        --form-primary: #3b82f6;
        --form-secondary: #8b5cf6;
        --form-success: #10b981;
        --form-error: #ef4444;
        --form-gradient-1: #3b82f6;
        --form-gradient-2: #8b5cf6;
    }
    
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: linear-gradient(135deg, var(--form-gradient-1), var(--form-gradient-2));
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .form-container {
        position: relative;
        width: 100%;
        max-width: 500px;
        margin: 0 20px;
    }
    
    .bg-decoration {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
        overflow: hidden;
    }
    
    .shape {
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        filter: blur(40px);
    }
    
    .shape-1 {
        width: 500px;
        height: 500px;
        top: -250px;
        right: -250px;
        animation: float 20s infinite ease-in-out;
    }
    
    .shape-2 {
        width: 300px;
        height: 300px;
        bottom: -150px;
        left: -150px;
        animation: float 20s infinite ease-in-out reverse;
    }
    
    @keyframes float {
        0%, 100% { transform: translate(0, 0) rotate(0deg); }
        50% { transform: translate(30px, -30px) rotate(180deg); }
    }
    
    .form-card, .success-card {
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(20px);
        border-radius: 24px;
        padding: 40px;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.15);
        position: relative;
        z-index: 1;
    }
    
    .success-card {
        text-align: center;
    }
    
    .success-icon {
        width: 80px;
        height: 80px;
        background: var(--form-success);
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 3rem;
        margin: 0 auto 20px;
        animation: scaleIn 0.5s ease;
    }
    
    @keyframes scaleIn {
        from {
            transform: scale(0);
            opacity: 0;
        }
        to {
            transform: scale(1);
            opacity: 1;
        }
    }
    
    header {
        text-align: center;
        margin-bottom: 30px;
    }
    
    h1 {
        font-size: 2.5rem;
        font-weight: 700;
        color: #1a202c;
        margin-bottom: 8px;
    }
    
    h2 {
        color: #1a202c;
        margin-bottom: 10px;
    }
    
    .gradient-text {
        background: linear-gradient(135deg, var(--form-primary), var(--form-secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }
    
    .subtitle {
        color: #64748b;
        font-size: 1.1rem;
    }
    
    form {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }
    
    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    
    label {
        font-weight: 600;
        color: #475569;
        font-size: 0.9rem;
    }
    
    input[type="text"],
    input[type="email"],
    input[type="password"],
    input[type="number"] {
        width: 100%;
        padding: 12px 16px;
        border: 2px solid #e2e8f0;
        border-radius: 10px;
        font-size: 1rem;
        transition: all 0.3s ease;
        background: white;
    }
    
    input:focus {
        outline: none;
        border-color: var(--form-primary);
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
    }
    
    .form-group.error input {
        border-color: var(--form-error);
    }
    
    .form-group.error input:focus {
        box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
    }
    
    .error-message {
        color: var(--form-error);
        font-size: 0.85rem;
        display: flex;
        align-items: center;
        gap: 4px;
    }
    
    .error-message::before {
        content: "!";
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
        background: var(--form-error);
        color: white;
        border-radius: 50%;
        font-size: 0.7rem;
        font-weight: bold;
    }
    
    .helper-text {
        color: #64748b;
        font-size: 0.85rem;
        min-height: 20px;
    }
    
    .checkbox-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    
    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        color: #475569;
        cursor: pointer;
    }
    
    input[type="checkbox"] {
        width: 20px;
        height: 20px;
        cursor: pointer;
        accent-color: var(--form-primary);
    }
    
    .checkbox-label a {
        color: var(--form-primary);
        text-decoration: none;
    }
    
    .checkbox-label a:hover {
        text-decoration: underline;
    }
    
    .btn {
        padding: 16px 32px;
        border: none;
        border-radius: 12px;
        font-size: 1.1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
    }
    
    .btn-primary {
        background: linear-gradient(135deg, var(--form-primary), var(--form-secondary));
        color: white;
        box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
    }
    
    .btn-primary:hover:not(.loading) {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
    }
    
    .btn.loading {
        opacity: 0.8;
        cursor: not-allowed;
    }
    
    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid rgba(255, 255, 255, 0.3);
        border-top-color: white;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    footer {
        text-align: center;
        margin-top: 30px;
        color: #64748b;
        font-size: 0.9rem;
    }
    
    footer a {
        color: var(--form-primary);
        text-decoration: none;
        font-weight: 600;
    }
    
    footer a:hover {
        text-decoration: underline;
    }
    
    @media (max-width: 600px) {
        h1 {
            font-size: 2rem;
        }
        
        .form-card {
            padding: 30px 20px;
        }
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Beautiful Form Validation starting...".into());
    mount(Box::new(BeautifulForm), "root");
    web_sys::console::log_1(&"Beautiful Form Validation mounted successfully!".into());
}