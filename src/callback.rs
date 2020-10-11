use crate::common::{find_1_field_with_attribute};
use crate::generics::{get_impl_block_generics, get_struct_block_generics, get_where_clause};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataStruct, Generics};

pub fn impl_callback(input: DataStruct, ident: Ident, generics: &Generics) -> TokenStream {
    let callback_field = match find_1_field_with_attribute(&input, "callback") {
        Ok(f) => f,
        Err(s) => return s,
    };

    let field_type = &callback_field.field.ty;
    let impl_block_generics = get_impl_block_generics(&generics);
    let struct_block_generics = get_struct_block_generics(&generics);
    let where_clause = get_where_clause(&generics);
    
    quote! {
        const _: () = {
            use teloxide::prelude::UpdateWithCx;
            use teloxide::contrib::callback::Callback;

            #[async_trait::async_trait]
            impl #impl_block_generics Callback for #ident #struct_block_generics #where_clause {
                type Update = <#field_type as Callback>::Update;
                type Err = <#field_type as Callback>::Err;

                async fn try_handle(&self, input: UpdateWithCx<Self::Update>)
                    -> Result<Result<(), Self::Err>, UpdateWithCx<Self::Update>>
                {
                    Callback::try_handle(&self.#callback_field, input).await
                }
            }
        };
    }
}
