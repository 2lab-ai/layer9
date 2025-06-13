//! Form field setter trait for dynamic field updates


/// Trait for types that can have their fields updated dynamically
pub trait FormFields {
    /// Set a field value by name
    fn set_field(&mut self, field: &str, value: String) -> Result<(), String>;
    
    /// Get a field value by name
    fn get_field(&self, field: &str) -> Option<String>;
    
    /// Get all field names
    fn field_names(&self) -> Vec<&'static str>;
}

/// Macro to implement FormFields for a struct
#[macro_export]
macro_rules! impl_form_fields {
    ($struct_name:ident { $($field:ident: $field_type:ty),* }) => {
        impl FormFields for $struct_name {
            fn set_field(&mut self, field: &str, value: String) -> Result<(), String> {
                match field {
                    $(
                        stringify!($field) => {
                            self.$field = value.parse::<$field_type>()
                                .map_err(|_| format!("Invalid value for field {}", field))?;
                            Ok(())
                        }
                    )*
                    _ => Err(format!("Unknown field: {}", field))
                }
            }
            
            fn get_field(&self, field: &str) -> Option<String> {
                match field {
                    $(
                        stringify!($field) => Some(self.$field.to_string()),
                    )*
                    _ => None
                }
            }
            
            fn field_names(&self) -> Vec<&'static str> {
                vec![$(stringify!($field)),*]
            }
        }
    };
}

/// Helper for string fields that don't need parsing
pub trait StringFormFields {
    fn set_string_field(&mut self, field: &str, value: String) -> Result<(), String>;
}

#[macro_export]
macro_rules! impl_string_form_fields {
    ($struct_name:ident { $($field:ident),* }) => {
        impl StringFormFields for $struct_name {
            fn set_string_field(&mut self, field: &str, value: String) -> Result<(), String> {
                match field {
                    $(
                        stringify!($field) => {
                            self.$field = value;
                            Ok(())
                        }
                    )*
                    _ => Err(format!("Unknown field: {}", field))
                }
            }
        }
    };
}