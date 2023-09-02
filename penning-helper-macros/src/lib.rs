use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn set_command(input: TokenStream, struct_data: TokenStream) -> TokenStream {
    let struct_data = syn::parse_macro_input!(struct_data as syn::ItemStruct);
    let name = &struct_data.ident;
    let command = input.to_string();
    let to_request_impl = quote::quote!{
        impl ToRequest for #name {
            const COMMAND: &'static str = #command;
        }
    };

    quote::quote! {
        #struct_data
        #to_request_impl
    }
    .into()
}
