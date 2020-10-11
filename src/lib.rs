mod view_factory;
mod callback;
mod common;
mod generics;
mod handler;
mod parser;
mod teloxide_attribute;
mod transition;

extern crate proc_macro;
extern crate quote;
extern crate syn;

use crate::common::compile_error;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

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
    teloxide_attribute::teloxide(attr, item)
}

/// The docs is below.
///
/// All the variants must be of the form `VariantName(MyStateType)`, and
/// `MyStateType` must implement `Subtransition`. All `MyStateType`s must have
/// the same `Subtransition::Aux` and `Subtransition::Error`, which will be also
/// used in the generated implementation.
#[proc_macro_derive(Transition)]
pub fn derive_transition(item: TokenStream) -> TokenStream {
    transition::derive_transition(item)
}

#[proc_macro_derive(Parser, attributes(parser))]
pub fn derive_parser_struct(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let s = match input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => return compile_error("Expected struct, found enum").into(),
        Data::Union(_) => return compile_error("Expected struct, found union").into(),
    };
    let res = parser::impl_parser(s, input.ident, input.generics);
    res.into()
}

#[proc_macro_derive(Callback, attributes(callback))]
pub fn derive_callback_struct(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let s = match input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => return compile_error("Expected struct, found enum").into(),
        Data::Union(_) => return compile_error("Expected struct, found union").into(),
    };
    let res = callback::impl_callback(s, input.ident, &input.generics);
    res.into()
}

#[proc_macro_derive(ViewFactory, attributes(view_factory))]
pub fn derive_view_factory_struct(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let s = match input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => return compile_error("Expected struct, found enum").into(),
        Data::Union(_) => return compile_error("Expected struct, found union").into(),
    };
    let res = view_factory::impl_view_factory(s, input.ident, &input.generics);
    res.into()
}
