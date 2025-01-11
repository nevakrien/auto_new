use syn::spanned::Spanned;
extern crate quote;

use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(new, attributes(new_visibility, include_arc))]
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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse visibility attribute
    let visibility = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("new_visibility"))
        .and_then(|attr| attr.parse_args::<syn::Visibility>().ok())
        .unwrap_or_else(|| syn::parse_quote!(pub)); // Default to `pub`

    // Check for `include_arc` attribute
    let include_arc = !input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_new_arc"));

    // Match on the type of fields in the struct
    let expanded = match struct_data.fields {
        Fields::Named(ref fields) => {
            // Handle named fields
            let new_args: Vec<_> = fields
                .named
                .iter()
                .map(|f| {
                    let name = f.ident.as_ref().unwrap(); // Named fields always have an identifier
                    let ty = &f.ty;
                    quote! { #name: #ty }
                })
                .collect();

            let init = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap();
                quote! { #name }
            });

            let arc_writes = fields.named.iter().enumerate().map(|(i, f)| {
                let name = f.ident.as_ref().unwrap();
                let ptr_name = syn::Ident::new(&format!("ptr{}", i), f.span());
                quote! {
                    let #ptr_name = &raw mut (*raw_mem.as_mut_ptr()).#name;
                    #ptr_name.write(#name);
                }
            });

            let new_arc_function = if include_arc {
                quote! {
                    #visibility fn new_arc(#(#new_args),*) -> std::sync::Arc<Self> {
                        unsafe {
                            let mut uninit: std::sync::Arc<std::mem::MaybeUninit<Self>> = std::sync::Arc::new_uninit();
                            let raw_mem: &mut std::mem::MaybeUninit<Self> = std::sync::Arc::get_mut(&mut uninit).unwrap();
                            #(#arc_writes)*
                            uninit.assume_init()
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    #visibility fn new(#(#new_args),*) -> Self {
                        Self {
                            #(#init),*
                        }
                    }

                    #new_arc_function
                }
            }
        }
        Fields::Unnamed(ref fields) => {
            // Handle tuple structs
            let new_args: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let arg_name = syn::Ident::new(&format!("arg{}", i), f.span());
                    let ty = &f.ty;
                    quote! { #arg_name: #ty }
                })
                .collect();

            let init = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
                quote! { #arg_name }
            });

            let arc_writes = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
                let ptr_name = syn::Ident::new(&format!("ptr{}", i), proc_macro2::Span::call_site());
                let idx = syn::Index::from(i);

                quote! {
                    let #ptr_name = &raw mut (*raw_mem.as_mut_ptr()).#idx;
                    #ptr_name.write(#arg_name);
                }
            });

            let new_arc_function = if include_arc {
                quote! {
                    #visibility fn new_arc(#(#new_args),*) -> std::sync::Arc<Self> {
                        unsafe {
                            let mut uninit: std::sync::Arc<std::mem::MaybeUninit<Self>> = std::sync::Arc::new_uninit();
                            let raw_mem: &mut std::mem::MaybeUninit<Self> = std::sync::Arc::get_mut(&mut uninit).unwrap();
                            #(#arc_writes)*
                            uninit.assume_init()
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    #visibility fn new(#(#new_args),*) -> Self {
                        Self(#(#init),*)
                    }

                    #new_arc_function
                }
            }
        }
        Fields::Unit => {
            // Handle unit structs
            let new_arc_function = if include_arc {
                quote! {
                    #visibility fn new_arc() -> std::sync::Arc<Self> {
                        std::sync::Arc::new(Self)
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    #visibility fn new() -> Self {
                        Self
                    }

                    #new_arc_function
                }
            }
        }
    };

    // Convert the generated code into a TokenStream
    TokenStream::from(expanded)
}
