//! Leptos-Sync Macros
//! 
//! This crate provides derive macros for automatic CRDT implementation
//! and local-first collection setup.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for automatic CRDT implementation
#[proc_macro_derive(LocalFirst, attributes(local_first))]
pub fn derive_local_first(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Implementation will be added here
    let name = input.ident;
    
    let expanded = quote! {
        impl LocalFirst for #name {
            // Auto-generated implementation
        }
    };
    
    TokenStream::from(expanded)
}
