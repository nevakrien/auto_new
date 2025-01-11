extern crate quote;

use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields,spanned::Spanned};

/// Automatically generates `new` and (optionaly) `new_arc` for a struct.
/// - **`new`**:
///   Creates a new instance of the struct from its components. 
///   Arguments are named and are ordered the same way the struct is.
///
/// - **`new_arc`** (optional):
///   same as `new` just creates an Arc\<MyStruct\>
/// 
/// # Usage
/// ```rust
///  use new_macro::new;
///
///  #[derive(new)]
///  struct MyStruct {
///      a: u32,
///      b: String,
///  }
///
///  fn main() {
///      let instance = MyStruct::new_arc(42, String::from("Hello, world!"));
///  }
/// ```
/// ### Attribute Options:
///
/// - **Custom Visibility**:
///  You can use the `#[new_visibility(...)]` attribute to limit the visibility of the generated functions.
///
/// ```rust
///  use new_macro::new;
///
///  #[derive(new)]
///  #[new_visibility(/*private*/)]
///  struct MyStruct (usize);
///
///  #[derive(new)]
///  #[new_visibility(pub crate)]
///  struct MyOtherStruct (usize);
///
///  ```
///
/// - **Exclude `new_arc`**:
///   Skip generating the `new_arc` function for environments that do not support
///   `std`, or when `Arc` is unnecessary, by using `#[no_new_arc]`.
///
///   ```rust
///   use new_macro::new;
///   
///   #[derive(new)]
///   #[no_new_arc]
///   struct MyStruct {
///       a: u32,
///   }
///   ```
///

#[proc_macro_derive(new, attributes(new_visibility, no_new_arc))]
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

    // Check for `no_new_arc` attribute
    let include_arc = !input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_new_arc"));

    // Generate functions
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

            /*
            while we could make new_arc using new thats going to be ineffishent in many cases. 
            because Arc::new(Something{}) usually first allocates on the stack and then moves to the heap. 
            this is dumb but the optimizer has a hard time seeing it because a heap allocation is a side effect. 
			to avoid this we explictly allocate the heap memory first and then write to it.
			*/
            let new_arc_function = if include_arc {

            	let arc_writes = fields.named.iter().enumerate().map(|(i, f)| {
	                let name = f.ident.as_ref().unwrap();
	                let ptr_name = syn::Ident::new(&format!("ptr{}", i), f.span());
	                quote! {
	                	// SAFETY: we know the memory is valid since its allocated in the lines above
	                    let #ptr_name = &raw mut (*raw_mem.as_mut_ptr()).#name;
	                    #ptr_name.write(#name);
	                }
	            });

                quote! {
                    #visibility fn new_arc(#(#new_args),*) -> std::sync::Arc<Self> {
                        // SAFETY: The raw memory is uninitialized but valid for writing because it is
			            // allocated by `Arc::new_uninit`. Each field is initialized exactly once
			            // before calling `assume_init`, ensuring the struct is fully initialized and nothing is forgoten.
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
            //Name arguments
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

            /*same notes as Named*/
            let new_arc_function = if include_arc {
            	let arc_writes = fields.unnamed.iter().enumerate().map(|(i, _)| {
	                let arg_name = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
	                let ptr_name = syn::Ident::new(&format!("ptr{}", i), proc_macro2::Span::call_site());
	                let idx = syn::Index::from(i);

	                quote! {
	                	// SAFETY: we know the memory is valid since its allocated in the lines above
	                    let #ptr_name = &raw mut (*raw_mem.as_mut_ptr()).#idx;
	                    #ptr_name.write(#arg_name);
	                }
	            });
                quote! {
                    #visibility fn new_arc(#(#new_args),*) -> std::sync::Arc<Self> {
                        // SAFETY: The raw memory is uninitialized but valid for writing because it is
			            // allocated by `Arc::new_uninit`. Each field is initialized exactly once
			            // before calling `assume_init`, ensuring the struct is fully initialized and nothing is forgoten.
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
        	//Units dont need anything fancy since there is no memory to move into the arc
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
