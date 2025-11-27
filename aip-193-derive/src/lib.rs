use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(status), supports(enum_any))]
struct StatusInput {
    ident: Ident,
    data: ast::Data<StatusVariant, ()>,
    domain: String,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(status))]
struct StatusVariant {
    ident: Ident,
    fields: ast::Fields<StatusField>,
    code: Ident,
    #[darling(default)]
    message: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(status))]
struct StatusField {
    ident: Option<Ident>,
    #[darling(default)]
    metadata: bool,
    #[darling(default)]
    metadata_key: Option<String>,
}

/// Intelligent detection crate path
///
/// Priority:
/// 1. `aip` (main crate) -> `::aip::__private::errors`
/// 2. `aip-193` (direct dependency) -> `::aip_193`
fn get_crate_path() -> proc_macro2::TokenStream {
    // Prioritize checking the main crate `aip`
    if let Ok(found) = crate_name("aip") {
        return match found {
            FoundCrate::Itself => quote!(crate::__private::errors),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, proc_macro2::Span::call_site());
                quote!(::#ident::__private::errors)
            }
        };
    }
    
    // Roll back to direct dependency on `aip-193`
    if let Ok(found) = crate_name("aip-193") {
        return match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, proc_macro2::Span::call_site());
                quote!(::#ident)
            }
        };
    }
    
    // Final rollback
    quote!(::aip_193)
}

#[proc_macro_derive(IntoStatus, attributes(status))]
pub fn derive_into_status(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let parsed = match StatusInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let expanded = generate_impl(&parsed);
    TokenStream::from(expanded)
}

fn generate_impl(input: &StatusInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let domain = &input.domain;
    let krate = get_crate_path();

    let variants = match &input.data {
        ast::Data::Enum(variants) => variants,
        _ => panic!("IntoStatus only supports enums"),
    };

    let code_arms = generate_code_arms(name, variants, &krate);
    let message_arms = generate_message_arms(name, variants);
    let metadata_arms = generate_metadata_arms(name, variants);

    quote! {
        impl #krate::IntoStatus for #name {
            fn code(&self) -> #krate::Code {
                match self {
                    #(#code_arms),*
                }
            }

            fn message(&self) -> ::std::string::String {
                match self {
                    #(#message_arms),*
                }
            }

            fn reason(&self) -> &str {
                self.as_ref()
            }

            fn domain(&self) -> &str {
                #domain
            }

            fn metadata(&self) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
                match self {
                    #(#metadata_arms),*
                }
            }
        }
    }
}

fn generate_code_arms(
    enum_name: &Ident, 
    variants: &[StatusVariant],
    krate: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    variants.iter().map(|v| {
        let code = &v.code;
        let pattern = generate_pattern(enum_name, v);
        quote! {
            #pattern => #krate::Code::#code
        }
    }).collect()
}

fn generate_message_arms(enum_name: &Ident, variants: &[StatusVariant]) -> Vec<proc_macro2::TokenStream> {
    variants.iter().map(|v| {
        let variant_name = &v.ident;
        let pattern = generate_pattern(enum_name, v);
        
        let message_expr = if let Some(template) = &v.message {
            quote! { format!(#template) }
        } else {
            let default_msg = format!("{}", variant_name);
            quote! { #default_msg.to_string() }
        };
        
        quote! {
            #pattern => #message_expr
        }
    }).collect()
}

fn generate_metadata_arms(enum_name: &Ident, variants: &[StatusVariant]) -> Vec<proc_macro2::TokenStream> {
    variants.iter().map(|v| {
        let pattern = generate_pattern(enum_name, v);
        
        let metadata_fields: Vec<_> = v.fields.iter()
            .filter(|f| f.metadata)
            .filter_map(|f| {
                let field_name = f.ident.as_ref()?;
                let key = f.metadata_key.clone()
                    .unwrap_or_else(|| field_name.to_string());
                Some(quote! {
                    map.insert(#key.to_string(), #field_name.to_string());
                })
            })
            .collect();

        quote! {
            #pattern => {
                #[allow(unused_mut)]
                let mut map = ::std::collections::HashMap::new();
                #(#metadata_fields)*
                map
            }
        }
    }).collect()
}

fn generate_pattern(enum_name: &Ident, variant: &StatusVariant) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    
    match &variant.fields.style {
        ast::Style::Unit => {
            quote! { #enum_name::#variant_name }
        }
        ast::Style::Struct => {
            let field_names: Vec<_> = variant.fields.iter()
                .filter_map(|f| f.ident.as_ref())
                .collect();
            quote! { #enum_name::#variant_name { #(#field_names),* } }
        }
        ast::Style::Tuple => {
            let bindings: Vec<_> = (0..variant.fields.len())
                .map(|i| {
                    let ident = Ident::new(&format!("_{}", i), proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            quote! { #enum_name::#variant_name(#(#bindings),*) }
        }
    }
}
