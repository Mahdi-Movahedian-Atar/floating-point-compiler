use proc_macro::TokenStream;
use std::collections::hash_map::DefaultHasher;
use quote::quote;
use std::hash::{Hash, Hasher};
use syn::{LitStr, parse_macro_input};

fn s_hash(string: &str) -> i16 {
    let mut hasher = DefaultHasher::new();

    string.hash(&mut hasher);

    let hash_value = hasher.finish() as i16;

    hash_value
}

#[proc_macro]
pub fn unique_i16(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr).value();

    let custom_hash_value = s_hash(&input_str);

    let result = quote! {
        #custom_hash_value
    };

    result.into()
}