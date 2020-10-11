use proc_macro::TokenStream;
use syn::{FnArg, ReturnType, parse_macro_input, ItemFn, Type};
use quote::quote;
use crate::common::compile_error;
use crate::handler::impl_handler;
use std::convert::identity;

pub fn teloxide(attr: TokenStream, item: TokenStream) -> TokenStream {
    match attr.to_string().as_ref() {
        "subtransition" => {
            let item_cloned = item.clone();
            let input = parse_macro_input!(item as ItemFn);
            let params = input.sig.inputs.iter().collect::<Vec<&FnArg>>();

            if params.len() != 2 && params.len() != 3 {
                panic!(
                    "An transition function must accept two/three parameters: \
                     a state type, TransitionIn, and an optional data."
                );
            }

            // This is actually used inside the quite! { ... } below.
            #[allow(unused_variables)]
                let state_type = match params[0] {
                FnArg::Typed(pat_type) => &pat_type.ty,
                _ => unreachable!(),
            };
            let fn_name = input.sig.ident;
            let fn_return_type = match input.sig.output {
                ReturnType::Type(_arrow, _type) => _type,
                _ => panic!(
                    "A subtransition must return TransitionOut<your dialogue \
                     type>"
                ),
            };
            let aux_param_type = match params.get(2) {
                Some(data_param_type) => match *data_param_type {
                    FnArg::Typed(typed) => typed.ty.clone(),
                    _ => unreachable!(),
                },
                None => {
                    let unit_type = proc_macro::TokenStream::from(quote! {()});
                    Box::new(parse_macro_input!(unit_type as Type))
                }
            };
            let call_fn = match params.get(2) {
                Some(_) => {
                    quote! {  #fn_name(self, cx, aux) }
                }
                None => quote! { #fn_name(self, cx) },
            };

            let item = proc_macro2::TokenStream::from(item_cloned);

            let impl_transition = quote! {
                impl teloxide::dispatching::dialogue::Subtransition for #state_type {
                    type Aux = #aux_param_type;
                    type Dialogue = <#fn_return_type as teloxide::dispatching::dialogue::SubtransitionOutputType>::Output;
                    type Error = <#fn_return_type as teloxide::dispatching::dialogue::SubtransitionOutputType>::Error;

                    fn react(self, cx: teloxide::dispatching::dialogue::TransitionIn, aux: #aux_param_type)
                        -> futures::future::BoxFuture<'static, #fn_return_type> {
                                #item
                                futures::future::FutureExt::boxed(#call_fn)
                            }
                }
            };

            impl_transition.into()
        }
        "handler" => {
            let input = parse_macro_input!(item as ItemFn);
            impl_handler(&input).unwrap_or_else(identity).into()
        }
        _ => {
            compile_error(format!("Unrecognised attribute '{}'", attr)).into()
        }
    }
}