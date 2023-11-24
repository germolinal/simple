//! # `SIMPLE` model derive macro
//!
//! The final behaviour of this macro should be relatively simple.
//! On the one hand, all `SIMPLE`-model objects—enums or structs—should be
//! * Representable in text format (i.e., readable by the scanner)
//! * Have a function that creates the documentation (used only for building automatic documentation)
//!
//! On the other hand, the SimulationState elements should be classifiable as either
//! * `operational` (is a window open?), `physical` (e.g., solar radiation over a wall) or `personal` (e.g., the amount of clothing weared by a person)
//!
//! This is handled by several macros.
//!
//! # Deriving Struct behaviour:
//!
//! There are two main kinds of fields in structs: `Optional` and `Mandatory`

use crate::simulation_state_behaviour::*;
use std::collections::HashMap;

fn get_attributes(ast: &DeriveInput) -> Vec<String> {
    let allowed_attributes = vec!["inline_enum".to_string()];

    ast.attrs
        .iter()
        .filter_map(|a| {
            if let Some(seg) = a.path.segments.iter().next() {
                let ident = format!("{}", seg.ident);
                if allowed_attributes.contains(&ident) {
                    Some(ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn object_location(typename: String) -> Option<&'static str> {
    let mapping = HashMap::from([
        ("Substance", "substances"),
        ("Material", "materials"),
        ("Construction", "constructions"),
        ("Surface", "surfaces"),
        ("Space", "spaces"),
        ("Building", "buildings"),
        ("Fenestration", "fenestrations"),
        ("HVAC", "hvacs"),
        ("Luminaire", "luminaires"),
        ("SiteDetails", "site_details"),
        ("Object", "objects"),
    ]);

    if let Some(v) = mapping.get(&typename.as_str()) {
        Some(v)
    } else {
        None
    }
}

fn object_has_api(typename: String) -> bool {
    let typename_bytes = typename.as_bytes();
    matches!(
        typename_bytes,
        b"Space" | b"Surface" | b"Fenestration" | b"HVAC" | b"Luminaire"
    )
}

mod common_path;
mod docs;
mod field;
mod object;
mod object_enum;
mod object_struct;
mod simulation_state_behaviour;

use crate::docs::get_docs;
use object::Object;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(StateElements, attributes(personal, operational, physical, references))]
pub fn derive_simulation_state_behaviour(input: TokenStream) -> TokenStream {
    let mut out = input.clone();

    let ast = parse_macro_input!(input as DeriveInput);
    let enum_name = &ast.ident;
    match ast.data {
        syn::Data::Enum(_) => {
            let variants = get_enum_variants(&ast);

            let derive_kind_variants = match derive_enum_kind(&ast, &variants) {
                Ok(s) => s,
                Err(e) => {
                    out.extend(TokenStream::from(e.to_compile_error()));
                    return out;
                }
            };

            let derive_output = match derive_output(&ast, &variants) {
                Ok(s) => s,
                Err(e) => {
                    out.extend(TokenStream::from(e.to_compile_error()));
                    return out;
                }
            };

            // Gather everything
            TokenStream::from(quote!(
                impl #enum_name {


                    #derive_kind_variants

                }

                #derive_output
            ))
        }
        _ => {
            panic!("SimulationStateBehaviour ::: can only be derived for Enums");
        }
    }
}

#[proc_macro_derive(ObjectIO, attributes(inline_enum))]
pub fn derive_input_output(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let attributes = get_attributes(&ast);
    let docs = get_docs(&ast.attrs).expect("Could not generate docs");
    let obj = Object::new(ast.clone(), docs, attributes);
    let object_name = &ast.ident;
    let name_str = format!("{}", object_name);

    // New
    let new = obj.gen_new().expect("Could not generate New");

    // name
    let name = obj.gen_name();

    // State getters and setters
    let state_getters_setters = obj
        .gen_state_getters_setters()
        .expect("Could not generate setters getters");

    // docs
    let docs = obj.gen_docs();

    let display = obj.gen_display();

    // return
    TokenStream::from(quote!(

        impl std::fmt::Display for #object_name{
            #display
        }


        impl #object_name {




            # docs

            /// Retrieves the type of object as a `&'static str`.
            ///
            /// This method is useful for debuging models that contain multiple objects
            pub fn object_type(&self) -> &str{
                #name_str
            }

            #name

            #new

            #state_getters_setters



        }
    ))
}

#[proc_macro_derive(GroupIO)]
pub fn derive_group_input_output(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attributes = get_attributes(&ast);
    let docs = get_docs(&ast.attrs).expect("Could not generate docs");
    let obj = Object::new(ast, docs, attributes);

    let q = obj.gen_group_behaviour();
    TokenStream::from(q)
}

#[proc_macro_derive(ObjectAPI, attributes(operational, physical))]
pub fn derive_object_api(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attributes = get_attributes(&ast);
    let docs = get_docs(&ast.attrs).expect("Could not generate docs");
    let obj = Object::new(ast, docs, attributes);
    TokenStream::from(obj.gen_object_api())
}

#[proc_macro_derive(GroupAPI)]
pub fn derive_group_api(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attributes = get_attributes(&ast);
    let docs = get_docs(&ast.attrs).expect("Could not generate API docs");
    let obj = Object::new(ast, docs, attributes);
    TokenStream::from(obj.gen_group_api())
}

#[proc_macro_derive(GroupMemberAPI, attributes(operational, physical))]
pub fn derive_group_member_api(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attributes = get_attributes(&ast);
    let docs = get_docs(&ast.attrs).expect("Could not generate API docs");
    let obj = Object::new(ast, docs, attributes);
    TokenStream::from(obj.gen_group_member_api())
}
