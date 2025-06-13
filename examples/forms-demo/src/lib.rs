use layer9_core::prelude::*;
use layer9_core::form_builder::FormBuilder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    let app = Layer9App::new("forms-demo");
    
    // Create a registration form
    let form = FormBuilder::new("registration")
        .text_field("username", "Username")
        .email_field("email", "Email")
        .password_field("password", "Password")
        .checkbox("terms", "I agree to terms")
        .submit_button("Register")
        .on_submit(|data| {
            web_sys::console::log_1(&format!("Form submitted: {:?}", data).into());
        })
        .build();
    
    app.mount(form);
}
