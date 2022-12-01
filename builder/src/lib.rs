use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = Ident::new(&format!("{name}Builder"), name.span());
    let builder_methods = generate_builder_methods(&input.data);
    let ast = quote! {
        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }
        impl #builder_name {
             #(#builder_methods)*
        }
        impl #name {
            fn builder() -> #builder_name {
                #builder_name {
                    executable: Default::default(),
                    args: Default::default(),
                    env: Default::default(),
                    current_dir: Default::default(),
                }
            }
        }

    }
    .into();
    eprintln!("TOKENS: {ast}");
    ast
}

fn generate_builder_methods(data: &Data) -> impl Iterator<Item = TokenStream> + '_ {
    let methods = match *data {
        syn::Data::Struct(ref s) => match &s.fields {
            Fields::Named(ref fields) => fields.named.iter().map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = Some(#name);
                        self
                    }
                }
            }),
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    methods
}
