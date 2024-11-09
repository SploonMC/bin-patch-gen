use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn serial(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
        #[serde(rename_all = "camelCase")]
        #input
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn serial_snake(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
        #input
    };

    TokenStream::from(expanded)
}