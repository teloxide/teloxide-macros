use quote::{ToTokens, quote};
use proc_macro2::TokenStream;

use syn::{Field, Fields, Token};
use syn::punctuated::Punctuated;

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

pub fn get_fields(fields: &Fields) -> Result<&Punctuated<Field, Token![,]>, TokenStream> {
    match fields {
        Fields::Named(named) => Ok(&named.named),
        Fields::Unnamed(unnamed) => Ok(&unnamed.unnamed),
        Fields::Unit => Err(compile_error("Expected struct with fields, found unit struct"))
    }
}
