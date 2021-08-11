use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Message)]
pub fn message_derive(item: TokenStream) -> TokenStream {
    let derive = syn::parse_macro_input!(item as DeriveInput);

    let name = &derive.ident;
    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();
    (quote! {
        impl #impl_generics ractor::message::Message for #name #ty_generics #where_clause {}
    }).into()
}
