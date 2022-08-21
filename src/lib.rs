// TODO: refactor this shit.

mod attr;
mod command;
mod command_enum;
mod dialogue_state;
mod fields_parse;
mod rename_rules;

extern crate proc_macro;
extern crate quote;
extern crate syn;
use crate::{
    attr::{Attr, VecAttrs},
    command::Command,
    command_enum::CommandEnum,
    fields_parse::{impl_parse_args_named, impl_parse_args_unnamed},
};
use attr::BotCommandAttribute;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Fields, ItemEnum};
use syn::{Lit::Str, Meta::NameValue, MetaNameValue};

#[proc_macro_derive(DialogueState, attributes(handler, handler_out, store))]
#[deprecated(note = "Use teloxide::handler! instead")]
pub fn derive_dialogue_state(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    match dialogue_state::expand(input) {
        Ok(s) => s.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

macro_rules! get_or_return {
    ($($some:tt)*) => {
        match $($some)* {
            Ok(elem) => elem,
            Err(e) => return e
        }
    }
}

#[proc_macro_derive(BotCommands, attributes(command))]
pub fn derive_telegram_command_enum(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let data_enum: &syn::DataEnum = get_or_return!(get_enum_data(&input));

    let enum_attrs: Vec<Attr> = get_or_return!(parse_attributes(&input.attrs));

    let command_enum = match CommandEnum::try_from(enum_attrs.as_slice()) {
        Ok(command_enum) => command_enum,
        Err(e) => return compile_error(e),
    };

    let variants: Vec<&syn::Variant> = data_enum.variants.iter().collect();

    let mut variant_infos = vec![];
    for variant in variants.iter() {
        let mut attrs = Vec::new();
        for attr in &variant.attrs {
            if attr.path.is_ident("doc") {
                // skip doc attr
            } else {
                match attr.parse_args::<VecAttrs>() {
                    Ok(mut attrs_) => {
                        attrs.append(attrs_.data.as_mut());
                    }
                    Err(e) => {
                        return compile_error(e.to_compile_error());
                    }
                }
            }
        }

        let comment_parts: Vec<_> = variant
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .filter_map(|attr| {
                if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) =
                    attr.parse_meta()
                {
                    Some(s.value())
                } else {
                    // non #[doc = "..."] attributes are not our concern
                    // we leave them for rustc to handle
                    None
                }
            })
            .collect();
        let doc = process_doc_comment(comment_parts);
        attrs.push(Attr { name: BotCommandAttribute::Description, value: doc });
        match Command::try_from(attrs.as_slice(), &variant.ident.to_string()) {
            Ok(command) => variant_infos.push(command),
            Err(e) => return compile_error(e),
        }
    }

    let mut vec_impl_create = vec![];
    for (variant, info) in variants.iter().zip(variant_infos.iter()) {
        let var = &variant.ident;
        let variantt = quote! { Self::#var };
        match &variant.fields {
            Fields::Unnamed(fields) => {
                let parser =
                    info.parser.as_ref().unwrap_or(&command_enum.parser_type);
                vec_impl_create
                    .push(impl_parse_args_unnamed(fields, variantt, parser));
            }
            Fields::Unit => {
                vec_impl_create.push(variantt);
            }
            Fields::Named(named) => {
                let parser =
                    info.parser.as_ref().unwrap_or(&command_enum.parser_type);
                vec_impl_create
                    .push(impl_parse_args_named(named, variantt, parser));
            }
        }
    }

    let ident = &input.ident;

    let fn_descriptions = impl_descriptions(&variant_infos, &command_enum);
    let fn_parse = impl_parse(&variant_infos, &command_enum, &vec_impl_create);
    let fn_commands = impl_commands(&variant_infos, &command_enum);

    let trait_impl = quote! {
        impl BotCommands for #ident {
            #fn_descriptions
            #fn_parse
            #fn_commands
        }
    };

    TokenStream::from(trait_impl)
}

fn impl_commands(
    infos: &[Command],
    global: &CommandEnum,
) -> quote::__private::TokenStream {
    let commands_to_list = infos.iter().filter_map(|command| {
        if command.description == Some("off".into()) {
            None
        } else {
            let c = command.get_matched_value(global);
            let d = command.description.as_deref().unwrap_or_default();
            Some(quote! { BotCommand::new(#c,#d) })
        }
    });
    quote! {
        fn bot_commands() -> Vec<teloxide::types::BotCommand> {
            use teloxide::types::BotCommand;
            vec![#(#commands_to_list),*]
        }
    }
}

fn impl_descriptions(
    infos: &[Command],
    global: &CommandEnum,
) -> quote::__private::TokenStream {
    let command_descriptions = infos.iter().filter_map(|c| {
        let (prefix, command) = c.get_matched_value2(global);
        let description = c.description.clone().unwrap_or_default();
        (description != "off").then(|| quote! { CommandDescription { prefix: #prefix, command: #command, description: #description } })
    });

    let global_description = match global.description.as_deref() {
        Some(gd) => quote! { .global_description(#gd) },
        None => quote! {},
    };

    quote! {
        fn descriptions() -> teloxide::utils::command::CommandDescriptions<'static> {
            use teloxide::utils::command::{CommandDescriptions, CommandDescription};
            use std::borrow::Cow;

            CommandDescriptions::new(&[
                #(#command_descriptions),*
            ])
            #global_description
        }
    }
}

fn impl_parse(
    infos: &[Command],
    global: &CommandEnum,
    variants_initialization: &[quote::__private::TokenStream],
) -> quote::__private::TokenStream {
    let matching_values = infos.iter().map(|c| c.get_matched_value(global));

    quote! {
         fn parse<N>(s: &str, bot_name: N) -> Result<Self, teloxide::utils::command::ParseError>
         where
              N: Into<String>
         {
              use std::str::FromStr;
              use teloxide::utils::command::ParseError;

              let mut words = s.splitn(2, ' ');
              let mut splited = words.next().expect("First item will be always.").split('@');
              let command_raw = splited.next().expect("First item will be always.");
              let bot = splited.next();
              let bot_name = bot_name.into();
              match bot {
                  Some(name) if name.eq_ignore_ascii_case(&bot_name) => {}
                  None => {}
                  Some(n) => return Err(ParseError::WrongBotName(n.to_string())),
              }
              let mut args = words.next().unwrap_or("").to_string();
              match command_raw {
                   #(
                        #matching_values => Ok(#variants_initialization),
                   )*
                   _ => Err(ParseError::UnknownCommand(command_raw.to_string())),
              }
         }
    }
}

fn get_enum_data(input: &DeriveInput) -> Result<&syn::DataEnum, TokenStream> {
    match &input.data {
        syn::Data::Enum(data) => Ok(data),
        _ => Err(compile_error("TelegramBotCommand allowed only for enums")),
    }
}

fn parse_attributes(
    input: &[syn::Attribute],
) -> Result<Vec<Attr>, TokenStream> {
    let mut enum_attrs = Vec::new();
    for attr in input.iter() {
        match attr.parse_args::<VecAttrs>() {
            Ok(mut attrs_) => {
                enum_attrs.append(attrs_.data.as_mut());
            }
            Err(e) => {
                return Err(compile_error(e.to_compile_error()));
            }
        }
    }
    Ok(enum_attrs)
}

fn compile_error<T>(data: T) -> TokenStream
where
    T: ToTokens,
{
    TokenStream::from(quote! { compile_error!(#data) })
}

// code from: https://github.com/clap-rs/clap/blob/453356e044db898d6ad4dd4578c2c50583913615/clap_derive/src/utils/doc_comments.rs#L11-L69
fn process_doc_comment(lines: Vec<String>) -> String {
    // multiline comments (`/** ... */`) may have LFs (`\n`) in them,
    // we need to split so we could handle the lines correctly
    //
    // we also need to remove leading and trailing blank lines
    let mut lines: Vec<&str> = lines
        .iter()
        .skip_while(|s| is_blank(s))
        .flat_map(|s| s.split('\n'))
        .collect();

    while let Some(true) = lines.last().map(|s| is_blank(s)) {
        lines.pop();
    }

    // remove one leading space no matter what
    for line in lines.iter_mut() {
        if line.starts_with(' ') {
            *line = &line[1..];
        }
    }

    lines.join("\n")
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}
