mod common;
mod parser;

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, ReturnType, Fields, FnArg, ItemFn, Type, ItemEnum};
use std::fmt::Write;
use crate::common::compile_error;

/// The docs is below.
///
/// The only accepted form at the current moment is `#[teloxide(subtransition)]`
/// on an asynchronous function. Either this:
///
/// ```no_compile
/// #[teloxide(subtransition)]
/// async fn my_transition(state: MyState, cx: TransitionIn, ans: T) -> TransitionOut<MyDialogue> {
///     todo!()
/// }
/// ```
///
/// Or this:
///
/// ```no_compile
/// #[teloxide(subtransition)]
/// async fn my_transition(state: MyState, cx: TransitionIn) -> TransitionOut<MyDialogue> {
///     todo!()
/// }
/// ```
///
/// Notice the presence/absence of `ans: T`. In the first case, it generates
/// `impl SubTransition for MyState { type Aux = T; type Dialogue = MyDialogue;
/// ... }`. In the second case, the `Aux` type defaults to `()`.
#[proc_macro_attribute]
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
        _ => {
            panic!("Unrecognised attribute '{}'", attr);
        }
    }
}

/// The docs is below.
///
/// All the variants must be of the form `VariantName(MyStateType)`, and
/// `MyStateType` must implement `Subtransition`. All `MyStateType`s must have
/// the same `Subtransition::Aux` and `Subtransition::Error`, which will be also
/// used in the generated implementation.
#[proc_macro_derive(Transition)]
pub fn derive_transition(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let mut dispatch_fn = "".to_owned();

    let enum_name = input.ident;
    let field_type_of_first_variant =
        match &input.variants.iter().next().unwrap().fields {
            Fields::Unnamed(fields) => {
                fields
                    .unnamed
                    .iter()
                    .next()
                    // .unwrap() because empty enumerations are not yet allowed
                    // in stable Rust.
                    .unwrap()
                    .ty
                    .to_token_stream()
                    .to_string()
            }
            _ => panic!("Only one unnamed field per variant is allowed"),
        };

    write!(
        dispatch_fn,
        "impl teloxide::dispatching::dialogue::Transition for {1} {{type Aux \
         = <{0} as teloxide::dispatching::dialogue::Subtransition>::Aux;type \
         Error = <{0} as \
         teloxide::dispatching::dialogue::Subtransition>::Error;fn \
         react(self, cx: teloxide::dispatching::dialogue::TransitionIn, aux: \
         Self::Aux) -> futures::future::BoxFuture<'static, \
         teloxide::dispatching::dialogue::TransitionOut<Self, Self::Error>> \
         {{ futures::future::FutureExt::boxed(async move {{ match self {{",
        field_type_of_first_variant, enum_name
    )
        .unwrap();

    for variant in input.variants.iter() {
        write!(
            dispatch_fn,
            "{}::{}(state) => \
             teloxide::dispatching::dialogue::Subtransition::react(state, cx, \
             aux).await,",
            enum_name, variant.ident
        )
            .unwrap();
    }

    write!(dispatch_fn, "}} }}) }} }}").unwrap();
    dispatch_fn.parse().unwrap()
}


#[proc_macro_derive(Parser, attributes(parser))]
pub fn derive_parser_struct(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let s = match input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => return compile_error("Expected struct, found enum").into(),
        Data::Union(_) => return compile_error("Expected struct, found union").into(),
    };
    let res = parser::impl_parser(s, input.ident);
    res.into()
}
