//! Form Handling with Validation - L5/L6

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

// Re-export form traits
pub use crate::form_traits::{FormFields, StringFormFields};

// Type aliases to simplify complex types
type ValidationFn<T> = Box<dyn Fn(&T) -> HashMap<String, Vec<String>>>;
type SubmitFn<T> = Box<dyn Fn(&T) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'static>>>;
type ValidatorFn = Box<dyn Fn(&str) -> Option<String>>;

/// Form state
#[derive(Clone)]
pub struct FormState<T> {
    pub values: T,
    pub errors: HashMap<String, Vec<String>>,
    pub touched: HashMap<String, bool>,
    pub submitting: bool,
    pub submitted: bool,
}

/// Form configuration
pub struct FormConfig<T> {
    pub initial_values: T,
    pub validate: Option<ValidationFn<T>>,
    pub on_submit: SubmitFn<T>,
}

/// Form hook
pub fn use_form<T: Clone + Default + 'static>(config: FormConfig<T>) -> Form<T> {
    let initial_values = config.initial_values.clone();
    let state = Rc::new(RefCell::new(FormState {
        values: initial_values,
        errors: HashMap::new(),
        touched: HashMap::new(),
        submitting: false,
        submitted: false,
    }));

    Form {
        state: state.clone(),
        config: Rc::new(config),
    }
}

/// Form handle
#[derive(Clone)]
pub struct Form<T> {
    state: Rc<RefCell<FormState<T>>>,
    config: Rc<FormConfig<T>>,
}

impl<T: Clone + 'static> Form<T> {
    pub fn values(&self) -> T {
        self.state.borrow().values.clone()
    }

    pub fn errors(&self) -> HashMap<String, Vec<String>> {
        self.state.borrow().errors.clone()
    }

    pub fn is_valid(&self) -> bool {
        self.state.borrow().errors.is_empty()
    }

    pub fn is_submitting(&self) -> bool {
        self.state.borrow().submitting
    }

    pub fn set_field_value(&self, field: &str, value: impl Into<String>) 
    where 
        T: FormFields
    {
        let value_string = value.into();
        let mut state = self.state.borrow_mut();
        
        // Update the field value
        if let Err(e) = state.values.set_field(field, value_string) {
            // If setting fails, add to errors
            state.errors.entry(field.to_string())
                .or_default()
                .push(e);
        } else {
            // Clear any previous errors for this field
            state.errors.remove(field);
            
            // Run validation if configured
            drop(state); // Release borrow before validate
            self.validate();
        }
    }

    pub fn set_field_touched(&self, field: &str, touched: bool) {
        self.state
            .borrow_mut()
            .touched
            .insert(field.to_string(), touched);
    }

    pub fn validate(&self) {
        if let Some(validate_fn) = &self.config.validate {
            let errors = validate_fn(&self.state.borrow().values);
            self.state.borrow_mut().errors = errors;
        }
    }

    pub fn handle_submit(&self) -> impl Fn() {
        let state = self.state.clone();
        let config = self.config.clone();

        move || {
            let mut state_mut = state.borrow_mut();
            state_mut.submitting = true;
            state_mut.submitted = true;

            // Validate all fields
            if let Some(validate_fn) = &config.validate {
                state_mut.errors = validate_fn(&state_mut.values);
            }

            if state_mut.errors.is_empty() {
                // Submit form
                let values = state_mut.values.clone();
                let state_clone = state.clone();
                let config = config.clone();

                spawn_local(async move {
                    match (config.on_submit)(&values).await {
                        Ok(_) => {
                            state_clone.borrow_mut().submitting = false;
                        }
                        Err(error) => {
                            state_clone.borrow_mut().submitting = false;
                            state_clone
                                .borrow_mut()
                                .errors
                                .insert("_form".to_string(), vec![error]);
                        }
                    }
                });
            } else {
                state_mut.submitting = false;
            }
        }
    }

    pub fn reset(&self) {
        *self.state.borrow_mut() = FormState {
            values: self.config.initial_values.clone(),
            errors: HashMap::new(),
            touched: HashMap::new(),
            submitting: false,
            submitted: false,
        };
    }
}

/// Validation rules
pub mod validators {
    use super::ValidatorFn;

    pub fn required(value: &str) -> Option<String> {
        if value.trim().is_empty() {
            Some("This field is required".to_string())
        } else {
            None
        }
    }

    pub fn email(value: &str) -> Option<String> {
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(value) {
            Some("Invalid email address".to_string())
        } else {
            None
        }
    }

    pub fn min_length(min: usize) -> impl Fn(&str) -> Option<String> {
        move |value: &str| {
            if value.len() < min {
                Some(format!("Must be at least {} characters", min))
            } else {
                None
            }
        }
    }

    pub fn max_length(max: usize) -> impl Fn(&str) -> Option<String> {
        move |value: &str| {
            if value.len() > max {
                Some(format!("Must be at most {} characters", max))
            } else {
                None
            }
        }
    }

    pub fn pattern(regex: &'static str, message: &'static str) -> impl Fn(&str) -> Option<String> {
        move |value: &str| {
            let re = regex::Regex::new(regex).unwrap();
            if !re.is_match(value) {
                Some(message.to_string())
            } else {
                None
            }
        }
    }

    pub fn compose(validators: Vec<ValidatorFn>) -> impl Fn(&str) -> Vec<String> {
        move |value: &str| {
            validators
                .iter()
                .filter_map(|validator| validator(value))
                .collect()
        }
    }
}

/// Form field components
pub struct TextField<T> {
    name: String,
    label: String,
    placeholder: Option<String>,
    form: Option<Form<T>>,
    value: String,
    error: Option<String>,
}

impl<T: Clone + FormFields + 'static> TextField<T> {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        TextField {
            name: name.into(),
            label: label.into(),
            placeholder: None,
            form: None,
            value: String::new(),
            error: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn bind(mut self, form: &Form<T>) -> Self {
        self.form = Some(form.clone());
        // Get initial value from form
        if let Some(value) = form.values().get_field(&self.name) {
            self.value = value;
        }
        // Get any errors
        if let Some(errors) = form.errors().get(&self.name) {
            self.error = errors.first().cloned();
        }
        self
    }
}

impl<T: Clone + FormFields + 'static> Component for TextField<T> {
    fn render(&self) -> Element {
        let form_clone = self.form.clone();
        let field_name = self.name.clone();
        
        let on_change = form_clone.map(|form| Rc::new(move |value: String| {
            form.set_field_value(&field_name, value);
            form.set_field_touched(&field_name, true);
        }) as Rc<dyn Fn(String)>);

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("form-field".to_string()),
                ..Default::default()
            },
            children: vec![
                Element::Node {
                    tag: "label".to_string(),
                    props: Props {
                        attributes: vec![("for".to_string(), self.name.clone())],
                        ..Default::default()
                    },
                    children: vec![Element::Text(self.label.clone())],
                },
                Element::Node {
                    tag: "input".to_string(),
                    props: Props {
                        class: Some("form-input".to_string()),
                        attributes: vec![
                            ("type".to_string(), "text".to_string()),
                            ("id".to_string(), self.name.clone()),
                            ("name".to_string(), self.name.clone()),
                            ("value".to_string(), self.value.clone()),
                            (
                                "placeholder".to_string(),
                                self.placeholder.as_deref().unwrap_or("").to_string(),
                            ),
                        ],
                        on_change,
                        ..Default::default()
                    },
                    children: vec![],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("form-error".to_string()),
                        ..Default::default()
                    },
                    children: if let Some(error) = &self.error {
                        vec![Element::Text(error.clone())]
                    } else {
                        vec![]
                    },
                },
            ],
        }
    }
}

/// Form field trait
#[allow(dead_code)]
trait FormField {
    fn get_value(&self) -> String;
    fn set_value(&mut self, value: String);
    fn get_errors(&self) -> Vec<String>;
}

#[allow(dead_code)]
struct EmptyFormField;
impl FormField for EmptyFormField {
    fn get_value(&self) -> String {
        String::new()
    }
    fn set_value(&mut self, _value: String) {}
    fn get_errors(&self) -> Vec<String> {
        vec![]
    }
}

/// Server actions
#[derive(Serialize, Deserialize)]
pub struct ServerAction<T, R> {
    pub action: String,
    pub data: T,
    _phantom: std::marker::PhantomData<R>,
}

impl<T: Serialize, R: for<'de> Deserialize<'de>> ServerAction<T, R> {
    pub fn new(action: impl Into<String>, data: T) -> Self {
        ServerAction {
            action: action.into(),
            data,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn execute(&self) -> Result<R, String> {
        let response = post("/api/actions", &self)
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;

        if response.ok() {
            response
                .json()
                .await
                .map_err(|e| format!("Parse error: {:?}", e))
        } else {
            Err(format!("Server error: {}", response.status()))
        }
    }
}

/// Form component with all fields
pub struct FormComponent<T> {
    form: Form<T>,
    children: Vec<Element>,
}

impl<T: Clone + 'static> FormComponent<T> {
    pub fn new(form: Form<T>) -> Self {
        FormComponent {
            form,
            children: vec![],
        }
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }
}

impl<T: Clone + 'static> Component for FormComponent<T> {
    fn render(&self) -> Element {
        let on_submit = self.form.handle_submit();

        Element::Node {
            tag: "form".to_string(),
            props: Props {
                on_submit: Some(Rc::new(move |_event| {
                    on_submit();
                })),
                ..Default::default()
            },
            children: self.children.clone(),
        }
    }
}

// Re-exports
use crate::fetch::post;
use wasm_bindgen_futures::spawn_local;
