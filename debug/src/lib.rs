use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index,
};


#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let fields = parse_fields(&input.data);
    let expanded = quote::quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(#name)
                #fields
                .finish()
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn parse_fields(data: &syn::Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote! {
                            .field(#name, &self.#name)
                        }
                    }).collect()
                },
                Fields::Unnamed(ref _fields) => {
                    quote! {}
                },
                Fields::Unit => {
                    quote!{}
                },
            }
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => unimplemented!(),
    }
}
