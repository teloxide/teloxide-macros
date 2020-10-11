use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use syn::punctuated::Punctuated;
use syn::{Field, Fields, Token, DataStruct};

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
        Fields::Unit => Err(compile_error(
            "Expected struct with fields, found unit struct",
        )),
    }
}

pub fn find_1_field_with_attribute<'a>(ds: &'a DataStruct, label: &str) -> Result<StructField<'a>, TokenStream> {
    let fields_has_attr = find_fields_with_attribute(ds)?;
    
    match fields_has_attr.as_slice() {
        [] => Err(compile_error(format!("One field must have `{}` attribute", label))),
        [x] => Ok(x.clone()),
        _ => Err(compile_error(format!("Only one field must have `{}` attribute", label))),
    }
}

pub fn find_fields_with_attribute(ds: &DataStruct) -> Result<Vec<StructField>, TokenStream> {
    let fields = get_fields(&ds.fields)?;
    let fields_has_attr = fields
        .iter()
        .enumerate()
        .filter(|(_, f)| is_have_attr(f))
        .map(|(num, f)| StructField::new(f, num))
        .collect::<Vec<_>>();
    Ok(fields_has_attr)
}

fn is_have_attr(field: &Field) -> bool {
    field.attrs.len() > 0
}
