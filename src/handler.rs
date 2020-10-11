use syn::{ItemFn, Signature, Generics, Type, FnArg, TypeReference, Path, PathArguments, PathSegment, GenericArgument, ReturnType};
use proc_macro2::{TokenStream, Ident};
use quote::{ToTokens, quote};
use crate::common::compile_error;
use crate::generics::{get_impl_block_generics, get_struct_block_generics, get_where_clause};

pub fn impl_handler(item: &ItemFn) -> Result<TokenStream, TokenStream> {
    validate_modifiers(&item.sig)?;
    let ImplInfo {
        fn_name, 
        generics, 
        struct_path, 
        data_type, 
        update_type, 
        err_type 
    } = parse_fn(&item)?;

    let impl_block_generics = get_impl_block_generics(&generics);
    let struct_block_generics = get_struct_block_generics(&generics);
    let where_clause = get_where_clause(&generics);
    
    Ok(quote! {
        #item
        #[async_trait::async_trait]
        impl #impl_block_generics Handler for #struct_path #struct_block_generics
            #where_clause
        {
            type Data = #data_type;
            type Update = #update_type;
            type Err = #err_type;
        
            async fn handle(
                &self, 
                data: DataWithUWC<Self::Data, Self::Update>
            ) -> Result<(), <Self as Handler>::Err> 
            {
                #fn_name(self, data).await
            }
        }
    })
}

fn validate_modifiers(sign: &Signature) -> Result<(), TokenStream> {
    if sign.constness.is_some() {
        return Err(compile_error("Cannot be const!"));
    }
    if sign.asyncness.is_none() {
        return Err(compile_error("Must be async!"));
    }
    if sign.unsafety.is_some() {
        return Err(compile_error("Cannot be unsafe!"));
    }
    if sign.abi.is_some() {
        return Err(compile_error("Cannot contain abi definition!"));
    }
    
    Ok(())
}

fn parse_fn(fun: &ItemFn) -> Result<ImplInfo, TokenStream> {
    let fn_name = &fun.sig.ident;
    let generics = &fun.sig.generics;
    let (struct_path, data_type, update_type) = match fun.sig.inputs.iter().collect::<Vec<_>>().as_slice() {
        [this, data] => {
            let struct_path = get_struct_path(this)?;
            let (data_type, update_type) = get_data_and_update_types(data)?;
            (struct_path, data_type, update_type)
        }
        _ => return Err(compile_error("Expected 2 args."))
    };
    let err_type = get_err_type(&fun.sig.output)?;
    Ok(ImplInfo {
        fn_name,
        generics,
        struct_path,
        data_type,
        update_type,
        err_type
    })
}

fn get_err_type(ret: &ReturnType) -> Result<&Type, TokenStream> {
    match ret {
        ReturnType::Type(_, t) => { 
            match t.as_ref() {
                Type::Path(p) => {
                    let segment = expect_1_path_segment(
                        &p.path, 
                        "Expected ident in return type, found path."
                    )?;

                    parse_return_type(segment)
                }
                _ => Err(compile_error("Expected ident in return type"))
            }
        },
        _ => Err(compile_error("Expected return type"))
    }
}

fn parse_return_type(segment: &PathSegment) -> Result<&Type, TokenStream> {
    if segment.ident != "Result" {
        return Err(compile_error(format!("Expected `Result` type in return type, found {}", segment.ident)))
    }
    let args = match &segment.arguments {
        PathArguments::AngleBracketed(gens) => gens,
        _ => return Err(compile_error(format!("Expected generics in return type")))
    };
    let (ok, err) = get_2_types_in_generics(args.args.iter().collect::<Vec<_>>().as_slice())?;
    validate_ok_type(ok)?;
    
    Ok(err)
}

fn validate_ok_type(ok: &Type) -> Result<(), TokenStream> {
    match ok {
        Type::Tuple(tup) => {
            match tup.elems.len() {
                0 => Ok(()),
                count => return Err(compile_error(
                    format!("Expected tuple with no args, found {} args", count)
                ))
            }
        }
        ty => return Err(compile_error(format!("Expected unit (), found {}", ty.into_token_stream())))
    }
}

fn get_struct_path(this: &FnArg) -> Result<&Path, TokenStream> {
    match this {
        FnArg::Receiver(_) => Err(compile_error("Expected arg, found self!")),
        FnArg::Typed(ty) => {
            match ty.ty.as_ref() {
                Type::Reference(tref) => {
                    validate_type_ref(tref)?;
                    match tref.elem.as_ref() {
                        Type::Path(p) => Ok(&p.path),
                        _ => Err(compile_error("Expected reference for the first argument!"))
                    }
                }
                _ => Err(compile_error("Expected reference for the first argument!"))
            }
        }
    }
}

fn get_data_and_update_types(arg: &FnArg) -> Result<(&Type, &Type), TokenStream> {
    match arg {
        FnArg::Receiver(_) => Err(compile_error("Expected arg, found self!")),
        FnArg::Typed(ty) => match ty.ty.as_ref() {
            Type::Path(path) => {
                let segment = expect_1_path_segment(
                    &path.path, 
                    "In second argument expected ident, found path."
                )?;
                get_data_and_update_types_in_segment(segment)
            }
            _ => Err(compile_error("Expected path in second argument."))
        }
    }
}

fn get_data_and_update_types_in_segment(segment: &PathSegment) -> Result<(&Type, &Type), TokenStream> {
    match segment.ident == "DataWithUWC" {
        true => {
            match &segment.arguments {
                PathArguments::AngleBracketed(generics) => {
                    get_2_types_in_generics(generics.args.iter().collect::<Vec<_>>().as_slice())
                }
                _ => Err(compile_error("In second argument expected ident with 2 generics."))
            }
        }
        false => Err(compile_error(
            format!("In second argument expected DataWithUWC, found {}.", segment.ident)
        )),
    }
}

fn get_2_types_in_generics<'a>(generics: &[&'a GenericArgument]) 
    -> Result<(&'a Type, &'a Type), TokenStream> 
{
    match generics {
        [data, update] => {
            let data_ty = expect_type(data)?;
            let update_ty = expect_type(update)?;
            Ok((data_ty, update_ty))
        }
        _ => Err(compile_error(
            format!("In second argument expected 2 generics, found {}.", generics.len())
        )),
    }
}

fn expect_type(generic: &GenericArgument) -> Result<&Type, TokenStream> {
    match generic {
        GenericArgument::Type(t) => Ok(t),
        _ => Err(compile_error("Expected type in generic!"))
    }
}

fn validate_type_ref(tref: &TypeReference) -> Result<(), TokenStream> {
    if tref.lifetime.is_some() {
        return Err(compile_error("Lifetimes is unallowed."))
    }
    if tref.mutability.is_some() {
        return Err(compile_error("Mutability is unallowed."))
    }
    
    Ok(())
}

struct ImplInfo<'a> {
    fn_name: &'a Ident,
    generics: &'a Generics,
    struct_path: &'a Path,
    data_type: &'a Type,
    update_type: &'a Type,
    err_type: &'a Type,
}

fn expect_1_path_segment<'a>(path: &'a Path, err: &'static str) -> Result<&'a PathSegment, TokenStream> {
    match path.segments.iter().collect::<Vec<_>>().as_slice() {
        [x] => {
            Ok(x)
        }
        _ => Err(compile_error(err))
    }
}
