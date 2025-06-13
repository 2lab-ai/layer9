//! Form builder utilities for easier form creation

use crate::form::{Form, FormConfig, FormFields};
use std::collections::HashMap;

type ValidatorFn = Box<dyn Fn(&str) -> Option<String>>;
type ValidatorsMap = HashMap<String, Vec<ValidatorFn>>;

/// Fluent form builder
pub struct FormBuilder<T> {
    initial_values: T,
    validators: ValidatorsMap,
}

impl<T: Clone + Default + 'static> Default for FormBuilder<T> {
    fn default() -> Self {
        Self {
            initial_values: T::default(),
            validators: HashMap::new(),
        }
    }
}

impl<T: Clone + Default + 'static> FormBuilder<T> {
    /// Create a new form builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set initial values
    pub fn with_initial_values(mut self, values: T) -> Self {
        self.initial_values = values;
        self
    }
    
    /// Add a field validator
    pub fn add_validator<F>(mut self, field: &str, validator: F) -> Self 
    where 
        F: Fn(&str) -> Option<String> + 'static
    {
        self.validators.entry(field.to_string())
            .or_default()
            .push(Box::new(validator));
        self
    }
    
    /// Build the form
    pub fn build<F>(self, on_submit: F) -> Form<T>
    where
        F: Fn(&T) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + 'static>> + 'static,
        T: FormFields,
    {
        let validators = self.validators;
        
        let validate = move |values: &T| -> HashMap<String, Vec<String>> {
            let mut errors = HashMap::new();
            
            for (field, field_validators) in &validators {
                if let Some(value) = values.get_field(field) {
                    let field_errors: Vec<String> = field_validators
                        .iter()
                        .filter_map(|validator| validator(&value))
                        .collect();
                    
                    if !field_errors.is_empty() {
                        errors.insert(field.clone(), field_errors);
                    }
                }
            }
            
            errors
        };
        
        crate::form::use_form(FormConfig {
            initial_values: self.initial_values,
            validate: Some(Box::new(validate)),
            on_submit: Box::new(on_submit),
        })
    }
}

/// Common validators
pub mod validators {
    /// Required field validator
    pub fn required(message: &str) -> impl Fn(&str) -> Option<String> {
        let msg = message.to_string();
        move |value: &str| {
            if value.trim().is_empty() {
                Some(msg.clone())
            } else {
                None
            }
        }
    }
    
    /// Email validator
    pub fn email(message: &str) -> impl Fn(&str) -> Option<String> {
        let msg = message.to_string();
        move |value: &str| {
            if value.contains('@') && value.contains('.') {
                None
            } else {
                Some(msg.clone())
            }
        }
    }
    
    /// Min length validator
    pub fn min_length(length: usize, message: &str) -> impl Fn(&str) -> Option<String> {
        let msg = message.to_string();
        move |value: &str| {
            if value.len() >= length {
                None
            } else {
                Some(msg.clone())
            }
        }
    }
    
    /// Max length validator
    pub fn max_length(length: usize, message: &str) -> impl Fn(&str) -> Option<String> {
        let msg = message.to_string();
        move |value: &str| {
            if value.len() <= length {
                None
            } else {
                Some(msg.clone())
            }
        }
    }
    
    /// Pattern validator
    pub fn pattern(regex: &str, message: &str) -> impl Fn(&str) -> Option<String> {
        let msg = message.to_string();
        let pattern = regex.to_string();
        move |value: &str| {
            // Simplified pattern matching - in production use regex crate
            if value.contains(&pattern) {
                None
            } else {
                Some(msg.clone())
            }
        }
    }
}