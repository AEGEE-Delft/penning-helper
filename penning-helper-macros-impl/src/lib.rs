use proc_macro2::{Ident, TokenStream};
use syn::parse::Parse;

#[deprecated = "This is for the old version of the Conscribo API"]
pub fn command(input: TokenStream, data: TokenStream) -> TokenStream {
    let struct_data = syn::parse2::<syn::ItemStruct>(data)
        .map_err(|e| e.to_compile_error())
        .unwrap();
    let name = &struct_data.ident;
    let (a, b, c) = struct_data.generics.split_for_impl();
    let Input {
        command_name,
        response_type,
    } = syn::parse2::<Input>(input).unwrap();
    let command_name = command_name.to_string();
    let to_request_impl = quote::quote! {
        impl #a ToRequest for #name #b #c {
            const COMMAND: &'static str = #command_name;

            type Response = #response_type;
        }
    };

    quote::quote! {
        #struct_data
        #to_request_impl
    }
    .into()
}

struct Input {
    command_name: Ident,
    response_type: syn::Type,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let command_name = input.parse()?;
        input.parse::<syn::Token![->]>()?;
        let response_type = input.parse()?;
        Ok(Self {
            command_name,
            response_type,
        })
    }
}

pub fn endpoint(input: TokenStream, _data: TokenStream) -> TokenStream {
    input
}