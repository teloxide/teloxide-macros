use crate::quote::ToTokens;
use proc_macro::TokenStream;
use std::fmt::Write;
use syn::Fields;
use syn::{parse_macro_input, ItemEnum};

pub fn derive_transition(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let mut dispatch_fn = "".to_owned();

    let enum_name = input.ident;
    let field_type_of_first_variant = match &input.variants.iter().next().unwrap().fields {
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
