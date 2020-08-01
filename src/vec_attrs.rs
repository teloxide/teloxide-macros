use syn::parse::{Parse, ParseStream};
use syn::Token;

pub struct VecAttrs<T: Parse> {
    pub data: Vec<T>,
}

impl<T: Parse> Parse for VecAttrs<T> {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut data = vec![];
        while !input.is_empty() {
            data.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { data })
    }
}