use quote::{ToTokens, quote};
use proc_macro2::TokenStream;

use syn::Field;

pub fn compile_error<T: ToTokens>(data: T) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#data);
    }
}

#[derive(Clone)]
pub struct StructField<'a> {
    pub field: &'a Field,
    number: usize,
}

impl<'a> StructField<'a> {
    pub fn new(field: &'a Field, number: usize) -> Self {
        StructField { field, number }
    }
}

impl ToTokens for StructField<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.field.ident {
            Some(id) => id.to_tokens(tokens),
            None => self.number.to_tokens(tokens),
        }
    }
}
