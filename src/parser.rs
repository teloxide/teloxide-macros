use syn::{DataStruct, Field, Generics};
use crate::common::{compile_error, StructField, get_fields};
use proc_macro2::{Ident, TokenStream, Span};
use quote::quote;
use crate::generics::{get_impl_block_generics, get_struct_block_generics, get_where_clause};

pub fn impl_parser(input: DataStruct, ident: Ident, generics: Generics) -> TokenStream {
    let parser_field = match find_parser_field(&input) {
        Ok(f) => f,
        Err(s) => return s,
    };
    
    let field_type = &parser_field.field.ty;
    let mod_ident = Ident::new(&format!("__private_parser_{}", ident), Span::call_site());
    let impl_block_generics = get_impl_block_generics(&generics);
    let struct_block_generics = get_struct_block_generics(&generics);
    let where_clause = get_where_clause(&generics);
    
    quote! {
        #[allow(proc_macro_derive_resolution_fallback)]
        mod #mod_ident {
            use teloxide::contrib::parser::DataWithUWC;
            use teloxide::prelude::UpdateWithCx;
            use teloxide::contrib::parser::Parser;
            use super::#ident;
            
            impl #impl_block_generics Parser for #ident #struct_block_generics #where_clause {
                type Update = <#field_type as Parser>::Update;
                type Output = <#field_type as Parser>::Output;
            
                fn parse(&self, data: UpdateWithCx<Self::Update>) -> Result<DataWithUWC<Self::Output, Self::Update>, UpdateWithCx<Self::Update>> {
                    Parser::parse(&self.#parser_field, data)
                }
            }
        }
    }
}

fn find_parser_field(ds: &DataStruct) -> Result<StructField, TokenStream> {
    let fields = get_fields(&ds.fields)?;
    let fields_has_parser = fields
        .iter()
        .enumerate()
        .filter(|(_, f)| is_have_parser_attr(f))
        .map(|(num, f)| StructField::new(f, num))
        .collect::<Vec<_>>();
    match fields_has_parser.as_slice() {
        [] => Err(compile_error("One field must have `parser` attribute")),
        [x] => Ok(x.clone()),
        _ => Err(compile_error("Only one field must have `parser` attribute")),
    }
}

fn is_have_parser_attr(field: &Field) -> bool {
    field.attrs.len() > 0 
}
