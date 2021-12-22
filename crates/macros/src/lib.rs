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

#![feature(let_else)]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput};

#[proc_macro_derive(DelegateEnum)]
pub fn delegate_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let tokens = implementation(&ast).unwrap_or_else(|err| err.to_compile_error());
    println!("{}", tokens);
    tokens.into()
}

fn implementation(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    println!("Name: {:?}", name);
    let Data::Enum(data) = &ast.data else {
        return Err(error("This macro only supports enums"));
    };

    for variant in &data.variants {
        println!("Identifier: {:?}", variant.ident)
    }

    Ok(TokenStream::new())
}

fn error(message: &str) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
