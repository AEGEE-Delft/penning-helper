use proc_macro::TokenStream;
use syn::{parse::Parse, Ident};

#[proc_macro_attribute]
pub fn set_command(input: TokenStream, struct_data: TokenStream) -> TokenStream {
    penning_helper_macros_impl::command(input.into(), struct_data.into()).into()
}

#[derive(Debug, PartialEq, Eq)]
enum AttrDescribe {
    Skip,
    Password,
    Email,
}

impl Parse for AttrDescribe {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input: Ident = input.parse()?;
        match input.to_string().as_str() {
            "skip" => Ok(AttrDescribe::Skip),
            "password" => Ok(AttrDescribe::Password),
            "email" => Ok(AttrDescribe::Email),
            m => panic!(
                "Needs to be in the format of #[describe(skip)] or #[describe(password)], was {}",
                m
            ),
        }
    }
}

impl AttrDescribe {
    fn from_meta(meta: &syn::Meta) -> Option<Self> {
        match meta {
            syn::Meta::List(list) => {
                if list.path.is_ident("describe") {
                    Some(list.parse_args().expect("Did not find expected types"))
                } else {
                    None
                }
            }
            a => panic!(
                "Needs to be in the format of #[describe(skip)] or #[describe(password)], was {:?}",
                a
            ),
        }
    }
}

#[proc_macro_derive(Describe, attributes(describe))]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let input = match input.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Describe can only be derived for structs"),
    };
    let fields = match input.fields {
        syn::Fields::Named(fields) => fields.named,
        _ => panic!("Describe can only be derived for structs with named fields"),
    };
    let m = fields
        .into_iter()
        .map(|f| {
            let attr = f
                .attrs
                .iter()
                .flat_map(|a| AttrDescribe::from_meta(&a.meta))
                .next();
            (f, attr)
        })
        .filter(|(_, attr)| !matches!(attr, Some(AttrDescribe::Skip)))
        .map(|(f, attr)| {
            (
                f.ident.unwrap().to_string(),
                f.ty,
                attr.unwrap_or(AttrDescribe::Skip),
            )
        })
        .map(|(ident, ty, attr)| match attr {
            AttrDescribe::Skip => quote::quote! {
                (#ident, <#ty as Describe>::describe_self()),
            },
            AttrDescribe::Password => quote::quote! {
                (#ident, Type::Password),
            },
            AttrDescribe::Email => quote::quote! {
                (#ident, Type::Email),
            },
        })
        .collect::<Vec<_>>();

    quote::quote! {
        impl Describe for #name {
            fn describe_fields() -> Vec<(&'static str, Type)> {
                vec![
                    #(#m)*
                ]
            }
        }
    }
    .into()
}
