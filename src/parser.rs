use crate::common::{find_1_field_with_attribute};
use crate::generics::{get_impl_block_generics, get_struct_block_generics, get_where_clause};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataStruct, Generics};

pub fn impl_parser(input: DataStruct, ident: Ident, generics: Generics) -> TokenStream {
    let parser_field = match find_1_field_with_attribute(&input, "parser") {
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
