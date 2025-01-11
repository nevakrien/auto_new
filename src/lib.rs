extern crate quote;

use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input,DeriveInput, Data,Fields };

#[proc_macro_derive(new)]
pub fn derive_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Ensure the macro is only used with structs
    let struct_data = match input.data {
        Data::Struct(s) => s,
        _ => {
            return syn::Error::new(input.ident.span(), "only structs are allowed with #[derive(new)]")
                .into_compile_error()
                .into();
        }
    };

    let struct_name = input.ident;
    let (impl_generics, ty_generics, where_clause) =  input.generics.split_for_impl();


    // Match on the type of fields in the struct
    let expanded = match struct_data.fields {
        Fields::Named(ref fields) => {
            // Handle named fields
            let new_args = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap(); // Named fields always have an identifier
                let ty = &f.ty;
                quote! { #name: #ty }
            });

            let init = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap();
                quote! { #name }
            });

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    pub fn new (#(#new_args),*) -> Self {
                        Self {
                            #(#init),*
                        }
                    }
                }
            }
        }
        Fields::Unnamed(ref fields) => {
		    // Handle tuple structs
		    let new_args = fields.unnamed.iter().enumerate().map(|(i, f)| {
		        let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
		        let ty = &f.ty;
		        quote! { #arg_name: #ty }
		    });

		    let init = fields.unnamed.iter().enumerate().map(|(i, _)| {
		        let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
		        quote! { #arg_name }
		    });

		    quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    pub fn new(#(#new_args),*) -> Self {
                        Self(#(#init),*)
                    }
                }
            }
		}
        Fields::Unit => {
            // Handle unit structs
            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    pub fn new() -> Self {
                        Self
                    }
                }
            }
        }
    };

    // Convert the generated code into a TokenStream
    TokenStream::from(expanded)
}
