//! Layer9 Procedural Macros

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct};

/// Macro for defining Layer9 apps
#[proc_macro_attribute]
pub fn layer9_app(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn new() -> Self {
                Self
            }
        }
    };

    expanded.into()
}

/// Macro for defining pages
#[proc_macro_attribute]
pub fn page(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let _args = proc_macro2::TokenStream::from(args);

    // For now, just pass through
    let expanded = quote! {
        #input
    };

    expanded.into()
}

/// Macro for defining components
#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let name = &input.sig.ident;

    let expanded = quote! {
        #input

        struct #name;

        impl Component for #name {
            fn render(&self) -> Element {
                #name()
            }
        }
    };

    expanded.into()
}

/// Macro for server functions
#[proc_macro_attribute]
pub fn server(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // For now, just pass through
    let expanded = quote! {
        #input
    };

    expanded.into()
}
