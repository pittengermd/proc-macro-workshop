use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::token::Comma;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder_name = Ident::new(&format!("{name}Builder"), name.span());
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!();
    };
    let builder_methods = generate_builder_methods(fields);
    let build_method = generate_build_method(&name, fields);
    let builder_fields = generate_builder_fields(fields);
    let ast = quote! {
        pub struct #builder_name {
            #(#builder_fields),*
        }
        impl #builder_name {
            #(#builder_methods)*
            #build_method
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

fn generate_builder_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if get_inner_ty("Option", ty).is_some() {
            quote! { #name: #ty }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    })
}

fn generate_builder_methods(
    fields: &syn::punctuated::Punctuated<syn::Field, Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let (arg_ty, value) = if let Some(inner_ty) = get_inner_ty("Option", ty) {
            (inner_ty, quote! { std::option::Option::Some(#name) })
        } else {
            (ty, quote! { std::option::Option::Some(#name) })
        };
        quote! {
            pub fn #name(&mut self, #name: #arg_ty) -> &mut Self {
                self.#name = #value;
                self
            }
        }
    })
}

//The requirements of this function state that every field must be checked that it was explicity set.
//However, with the CommandBuilder struct defined as it,
//there is no way to differiate between a field being
//explicity declared as None and it not being set yet.
//This is a problem.
fn generate_build_method(
    name: &Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, Comma>,
) -> TokenStream {
    let build_impl = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let error_msg = format!("{name:#?} is missing");
        if get_inner_ty("Option", ty).is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.as_ref().ok_or(#error_msg)?.clone(),
            }
        }
    });
    quote! {
        pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
            Ok(#name {
                #(#build_impl)*
            })
        }
    }
}

//This function is icky and too many nested if lets.  This should be simplified
// to check for the only condition I care about and return some then, else return None.
//Should be a single if statement
fn get_inner_ty<'a>(outer: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != outer {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(inner_ty) = &p.path.segments[0].arguments {
            if inner_ty.args.len() != 1 {
                return None;
            }
            
            if let Some(ab) = inner_ty.args.first() {
                match ab {
                    syn::GenericArgument::Type(gty) => return Some(gty),
                    syn::GenericArgument::Lifetime(_)
                    | syn::GenericArgument::Binding(_)
                    | syn::GenericArgument::Constraint(_)
                    | syn::GenericArgument::Const(_) => return None,
                }
            }
        }
    }
    None
}
