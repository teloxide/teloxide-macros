use syn::{DataStruct, Field};
use crate::common::{compile_error, StructField, get_fields};
use proc_macro2::{Ident, TokenStream, Span};
use quote::quote;

pub fn impl_callback(input: DataStruct, ident: Ident) -> TokenStream {
    let callback_field = match find_callback_field(&input) {
        Ok(f) => f,
        Err(s) => return s,
    };

    let field_type = &callback_field.field.ty;
    let mod_ident = Ident::new(&format!("__private_callback_{}", ident), Span::call_site());

    // TODO: add support for generics
    quote! {
        #[allow(proc_macro_derive_resolution_fallback)]
        mod #mod_ident {
            use teloxide::contrib::parser::DataWithUWC;
            use teloxide::prelude::UpdateWithCx;
            use teloxide::contrib::callback::Callback;
            use super::#ident;
            
            #[async_trait::async_trait]
            impl Callback for #ident {
                type Update = <#field_type as Callback>::Update;
                type Err = <#field_type as Callback>::Err;
            
                async fn try_handle(&self, input: UpdateWithCx<Self::Update>) 
                    -> Result<Result<(), Self::Err>, UpdateWithCx<Self::Update>> 
                {
                    Callback::try_handle(&self.#callback_field, input).await
                }
            }
        }
    }
}

fn find_callback_field(ds: &DataStruct) -> Result<StructField, TokenStream> {
    let fields = get_fields(&ds.fields)?;
    let fields_has_parser = fields
        .iter()
        .enumerate()
        .filter(|(_, f)| is_have_callback_attr(f))
        .map(|(num, f)| StructField::new(f, num))
        .collect::<Vec<_>>();
    match fields_has_parser.as_slice() {
        [] => Err(compile_error("One field must have `callback` attribute")),
        [x] => Ok(x.clone()),
        _ => Err(compile_error("Only one field must have `callback` attribute")),
    }
}

fn is_have_callback_attr(field: &Field) -> bool {
    field.attrs.len() > 0
}
