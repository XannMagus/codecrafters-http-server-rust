use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromString)]
pub fn derive_from_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        struct _AssertFromStrImplemented
        where
            #name: From<&'static str>;

        impl From<String> for #name {
            fn from(s: String) -> Self {
                Self::from(s.as_str())
            }
        }

        impl From<&String> for #name {
            fn from(s: &String) -> Self {
                Self::from(s.as_str())
            }
        }
    };

    TokenStream::from(expanded)
}