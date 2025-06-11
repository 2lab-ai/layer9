//! Layer9 Counter Example - Minimal Version

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

// Thread-safe counter state
thread_local! {
    static COUNTER: RefCell<i32> = const { RefCell::new(0) };
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages in browser
    console_error_panic_hook::set_once();

    // Get document and body
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    // Clear body
    body.set_inner_html("");

    // Create container
    let container = create_element(&document, "div")?;
    container.set_class_name("layer9-app");

    // Create title
    let title = create_element(&document, "h1")?;
    title.set_text_content(Some("Layer9 Counter"));
    container.append_child(&title)?;

    // Create counter display
    let counter_display = create_element(&document, "p")?;
    counter_display.set_id("counter-display");
    update_counter_display(&document)?;
    container.append_child(&counter_display)?;

    // Create button container
    let button_container = create_element(&document, "div")?;
    button_container.set_class_name("button-container");

    // Create increment button
    let inc_button = create_element(&document, "button")?;
    inc_button.set_text_content(Some("Increment"));
    inc_button.set_class_name("btn btn-primary");

    // Add click handler
    let inc_closure = Closure::wrap(Box::new(move || {
        COUNTER.with(|counter| {
            *counter.borrow_mut() += 1;
        });
        let doc = web_sys::window().unwrap().document().unwrap();
        update_counter_display(&doc).unwrap();
    }) as Box<dyn Fn()>);

    inc_button.add_event_listener_with_callback("click", inc_closure.as_ref().unchecked_ref())?;
    inc_closure.forget();

    // Create decrement button
    let dec_button = create_element(&document, "button")?;
    dec_button.set_text_content(Some("Decrement"));
    dec_button.set_class_name("btn btn-secondary");

    let dec_closure = Closure::wrap(Box::new(move || {
        COUNTER.with(|counter| {
            *counter.borrow_mut() -= 1;
        });
        let doc = web_sys::window().unwrap().document().unwrap();
        update_counter_display(&doc).unwrap();
    }) as Box<dyn Fn()>);

    dec_button.add_event_listener_with_callback("click", dec_closure.as_ref().unchecked_ref())?;
    dec_closure.forget();

    // Create reset button
    let reset_button = create_element(&document, "button")?;
    reset_button.set_text_content(Some("Reset"));
    reset_button.set_class_name("btn btn-warning");

    let reset_closure = Closure::wrap(Box::new(move || {
        COUNTER.with(|counter| {
            *counter.borrow_mut() = 0;
        });
        let doc = web_sys::window().unwrap().document().unwrap();
        update_counter_display(&doc).unwrap();
    }) as Box<dyn Fn()>);

    reset_button
        .add_event_listener_with_callback("click", reset_closure.as_ref().unchecked_ref())?;
    reset_closure.forget();

    // Add buttons to container
    button_container.append_child(&inc_button)?;
    button_container.append_child(&dec_button)?;
    button_container.append_child(&reset_button)?;
    container.append_child(&button_container)?;

    // Add some info
    let info = create_element(&document, "p")?;
    info.set_class_name("info");
    info.set_inner_html("Built with <strong>Layer9</strong> - A Rust Web Framework");
    container.append_child(&info)?;

    // Add container to body
    body.append_child(&container)?;

    // Add some basic styles
    add_styles(&document)?;

    // Log success
    web_sys::console::log_1(&"Layer9 Counter App initialized!".into());

    Ok(())
}

fn create_element(document: &Document, tag: &str) -> Result<Element, JsValue> {
    document.create_element(tag)
}

fn update_counter_display(document: &Document) -> Result<(), JsValue> {
    let display = document.get_element_by_id("counter-display").unwrap();
    COUNTER.with(|counter| {
        display.set_text_content(Some(&format!("Count: {}", *counter.borrow())));
    });
    Ok(())
}

fn add_styles(document: &Document) -> Result<(), JsValue> {
    let style = document.create_element("style")?;
    style.set_text_content(Some(
        r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        
        .layer9-app {
            max-width: 600px;
            margin: 0 auto;
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
        }
        
        h1 {
            color: #333;
            margin-bottom: 20px;
        }
        
        #counter-display {
            font-size: 48px;
            font-weight: bold;
            color: #007bff;
            margin: 30px 0;
        }
        
        .button-container {
            display: flex;
            gap: 10px;
            justify-content: center;
            margin: 20px 0;
        }
        
        .btn {
            padding: 10px 20px;
            font-size: 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            transition: all 0.3s ease;
        }
        
        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 2px 5px rgba(0,0,0,0.2);
        }
        
        .btn-primary {
            background-color: #007bff;
            color: white;
        }
        
        .btn-secondary {
            background-color: #6c757d;
            color: white;
        }
        
        .btn-warning {
            background-color: #ffc107;
            color: #212529;
        }
        
        .info {
            margin-top: 40px;
            color: #666;
            font-size: 14px;
        }
        
        .info strong {
            color: #007bff;
        }
    "#,
    ));

    let head = document.query_selector("head")?.unwrap();
    head.append_child(&style)?;
    Ok(())
}
