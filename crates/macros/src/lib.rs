// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Macros used throughout the project

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Fields, GenericArgument, Path,
    PathArguments, Type,
};

/// Generates Event & Query structs for the delegates in the delegate enum.
/// See the module comment in `delegates.rs` for more information about this
/// system.
#[proc_macro_derive(DelegateEnum)]
pub fn delegate_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let tokens = implementation(&ast).unwrap_or_else(|err| err.to_compile_error());
    tokens.into()
}

fn implementation(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let parsed = parse(ast);
    generated(parsed?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DelegateType {
    Event,
    Query,
}

#[derive(Debug, Clone)]
struct ParsedVariant {
    name: Ident,
    docs: Vec<Attribute>,
    data: Path,
    output: Option<Path>,
    delegate_type: DelegateType,
}

fn parse(ast: &DeriveInput) -> syn::Result<Vec<ParsedVariant>> {
    let data = match &ast.data {
        Data::Enum(d) => d,
        _ => return Err(error("Expected enum")),
    };

    let mut result = vec![];
    for variant in &data.variants {
        let fields = match &variant.fields {
            Fields::Unnamed(f) => f,
            _ => return Err(error("Expected unnamed field")),
        };

        let docs = variant
            .attrs
            .iter()
            .filter(|attribute| attribute.path.is_ident("doc"))
            .cloned()
            .collect();
        let field = fields.unnamed.iter().next().ok_or_else(|| error("Expected a field"))?;
        let path = match &field.ty {
            Type::Path(p) => p,
            _ => return Err(error("Expected path")),
        };

        let segment =
            &path.path.segments.iter().next().ok_or_else(|| error("Expected a path segment"))?;
        let args = match &segment.arguments {
            PathArguments::AngleBracketed(a) => a,
            _ => return Err(error("Expected PathArguments::AngleBracketed")),
        };

        let delegate_type = if segment.ident == "QueryDelegate" {
            DelegateType::Query
        } else {
            DelegateType::Event
        };

        result.push(ParsedVariant {
            name: variant.ident.clone(),
            docs,
            data: generic_argument(args, 0)?.clone(),
            output: if delegate_type == DelegateType::Event {
                None
            } else {
                Some(generic_argument(args, 1)?.clone())
            },
            delegate_type,
        });
    }

    Ok(result)
}

fn generated(variants: Vec<ParsedVariant>) -> syn::Result<TokenStream> {
    let variants = variants.iter().map(generate_variant);
    Ok(quote! {
        #(#variants)*
    })
}

fn generate_variant(variant: &ParsedVariant) -> impl ToTokens {
    let name = &variant.name;
    let struct_name = format_ident!(
        "{}{}",
        name,
        if variant.delegate_type == DelegateType::Event { "Event" } else { "Query" }
    );
    let docs = &variant.docs;
    let data = &variant.data;

    let (trait_value, return_value) = if variant.delegate_type == DelegateType::Event {
        (quote! {EventData<#data>}, quote! {Option<&EventDelegate<#data>>})
    } else {
        let output = variant.output.as_ref().unwrap();
        (quote! {QueryData<#data, #output>}, quote! {Option<&QueryDelegate<#data, #output>>})
    };

    quote! {
        #(#docs)*
        #[derive(Debug, Copy, Clone)]
        pub struct #struct_name(pub #data);

        impl #trait_value for #struct_name {
            fn data(&self) -> #data {
                self.0
            }

            fn kind(&self) -> DelegateKind {
                DelegateKind::#name
            }

            fn extract(delegate: &Delegate) -> #return_value {
                match delegate {
                    Delegate::#name(d) => Some(d),
                    _ => None,
                }
            }
        }
    }
}

fn generic_argument(input: &AngleBracketedGenericArguments, index: usize) -> syn::Result<&Path> {
    let arg =
        input.args.iter().nth(index).ok_or_else(|| error("Missing expected generic parameter"))?;
    let path = match arg {
        GenericArgument::Type(Type::Path(p)) => p,
        _ => return Err(error("Expected GenericArgument::Type")),
    };
    Ok(&path.path)
}

fn error(message: &str) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
