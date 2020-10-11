use crate::common::{find_1_field_with_attribute};
use crate::generics::{get_impl_block_generics, get_struct_block_generics, get_where_clause};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataStruct, Generics};

pub fn impl_view_factory(input: DataStruct, ident: Ident, generics: &Generics) -> TokenStream {
    let parser_field = match find_1_field_with_attribute(&input, "view_factory") {
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
            use teloxide::contrib::views::ViewFactory;
            use super::#ident;

            impl #impl_block_generics ViewFactory for #ident #struct_block_generics #where_clause
            {
                type Ctx = <#field_type as ViewFactory>::Ctx;
                type View = <#field_type as ViewFactory>::View;
                
                fn construct(&self, ctx: Self::Ctx) -> Self::View {
                    ViewFactory::construct(&self.#parser_field, ctx)
                }
            }
        }
    }
}
