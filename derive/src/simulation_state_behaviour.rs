/*
MIT License
Copyright (c)  Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;

pub fn get_enum_variants(
    ast: &syn::DeriveInput,
) -> syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> {
    if let syn::Data::Enum(syn::DataEnum { ref variants, .. }) = ast.data {
        variants.clone()
    } else {
        panic!("THIS IS A BUG: Expecting an Enum");
    }
}

pub fn contains_attr(v: &syn::Variant, att: &str) -> bool {
    v.attrs
        .iter()
        .map(|a| format!("{}", a.path().segments[0].ident))
        .any(|x| x == *att)
}

pub fn derive_enum_kind(
    ast: &syn::DeriveInput,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Result<TokenStream2, syn::Error> {
    let enum_name = &ast.ident;

    // Check that each variant has one and only one category
    for v in variants {
        let att_names: Vec<String> = v
            .attrs
            .iter()
            .map(|a| format!("{}", a.path().segments[0].ident))
            .collect();
        let mut n = 0;
        if att_names.contains(&"physical".to_string()) {
            n += 1;
        }
        if att_names.contains(&"personal".to_string()) {
            n += 1;
        }
        if att_names.contains(&"operational".to_string()) {
            n += 1;
        }
        if n == 0 {
            return Err(syn::Error::new(v.span(),format!("Variant {} has no attributes... must have either 'physical', 'personal' or 'operational'", v.ident)));
        }
        if n > 1 {
            return Err(syn::Error::new(v.span(),format!("Variant {} has too many attributes... it must have ONLY ONE between 'physical', 'personal' or 'operational'", v.ident)));
        }
    }

    // Sort variants
    let each_physical_variant = variants
        .iter()
        .filter(|v| contains_attr(v, "physical"))
        .map(|x| {
            let v_ident = x.ident.clone();
            quote! {
                Self::#v_ident{..}
            }
        });

    let each_personal_variant = variants
        .iter()
        .filter(|v| contains_attr(v, "personal"))
        .map(|x| {
            let v_ident = x.ident.clone();
            quote! {
                Self::#v_ident{..}
            }
        });
    let each_operational_variant = variants
        .iter()
        .filter(|v| contains_attr(v, "operational"))
        .map(|x| {
            let v_ident = x.ident.clone();
            quote! {
                Self::#v_ident{..}
            }
        });

    let is_personal_docstring =
        format!(" Checks whether a [`{}`] is of kind `Personal`", enum_name);
    let is_physical_docstring =
        format!(" Checks whether a [`{}`] is of kind `Physical`", enum_name);
    let is_operational_docstring = format!(
        " Checks whether a [`{}`] is of kind `Operational`",
        enum_name
    );

    Ok(quote!(
        // impl #enum_name {
            #[doc = #is_physical_docstring]
            pub fn is_physical (self: &'_ Self) -> ::core::primitive::bool{
                match self {
                    #(
                        | #each_physical_variant => true,
                    )*
                    | _ => false,

                }
            }

            #[doc = #is_personal_docstring]
            pub fn is_personal (self: &'_ Self) -> ::core::primitive::bool{
                match self {
                    #(
                        | #each_personal_variant => true,
                    )*
                    | _ => false,

                }
            }

            #[doc = #is_operational_docstring]
            pub fn is_operational (self: &'_ Self) -> ::core::primitive::bool{
                match self {
                    #(
                        | #each_operational_variant => true,
                    )*
                    | _ => false,

                }
            }
        // }
    ))
}

fn sanitize_docs(docs: &str) -> TokenStream2 {
    let mut clean_docs = quote!();
    docs.lines().for_each(|ln| {
        let ln = ln.trim();
        clean_docs = quote!(
            #clean_docs

            #[doc=#ln]
        )
    });
    clean_docs
}

pub fn derive_output(
    ast: &syn::DeriveInput,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Result<TokenStream2, syn::Error> {
    let enum_name = &ast.ident;

    let mut output_enum = quote!();
    let mut get_output_string = quote!();

    // Check that each variant has one and only one category
    for v in variants {
        // Key info: identifier
        let v_ident = v.ident.clone();

        // Get all the attributes
        let mut points_to = None;
        for a in v.attrs.iter() {
            let a_name = a.path().segments[0].ident.to_string();

            if a_name == "references" {
                points_to = Some(crate::field::Attribute::new(a));
                break;
            }
        }

        // Extract the docs
        let v_doc_str = crate::docs::get_docs(&v.attrs).expect("Could not generate docs");
        let v_doc = sanitize_docs(&v_doc_str);

        // Add to enum
        output_enum = quote!(
            #output_enum
            #v_doc
            #v_ident (String),
        );

        // Add method to get string.

        let op = if let Some(p) = points_to {
            if let Some(value) = &p.value {
                let aux: Vec<(TokenStream2, String, TokenStream2)> = value
                    .split(',')
                    .enumerate()
                    .map(|(i, x)| {
                        let mut x = x.trim();
                        if x.starts_with('\"') {
                            x = x.strip_prefix('\"').expect("Unreachable?");
                        }
                        if x.ends_with('\"') {
                            x = x.strip_suffix('\"').expect("Unreachable?");
                        }
                        let varname = syn::Ident::new(&format!("in{}", i), Span::call_site());
                        let x = x.to_string();
                        let op = match crate::object_location(x) {
                            Some(loc) => {
                                let location = syn::Ident::new(loc, Span::call_site());
                                quote!(model.#location[*#varname].name())
                            }
                            None => quote!(#varname),
                        };

                        (quote!(#varname), "{}".to_string(), op)
                    })
                    .collect();

                let mut varnames = quote!();
                let mut ops = quote!();
                aux.iter().enumerate().for_each(|(i, x)| {
                    let name = x.0.clone();
                    let o = x.2.clone();
                    if i == 0 {
                        varnames = quote!(#name);
                        ops = quote!(#o);
                    } else {
                        varnames = quote!(#varnames, #name);
                        ops = quote!(#ops, #o);
                    }
                });

                let args: Vec<String> = aux.iter().map(|x| x.1.to_string()).collect();

                let args = args.join("-");
                (varnames, args, ops)
            } else {
                (quote!(), "".to_string(), quote!())
            }
        } else {
            (quote!(), "".into(), quote!())
        };

        let varnames = op.0;
        let args = op.1;
        let ops = op.2;

        let aux = format!("{{{{\"{}\":\"{}\"}}}}", v_ident, args);
        if args.is_empty() {
            get_output_string = quote!(

                #get_output_string
                #enum_name::#v_ident {..} => {
                    format!(#aux, #ops)
                },
            )
        } else {
            get_output_string = quote!(

                #get_output_string
                #enum_name::#v_ident(#varnames) => {
                    format!(#aux, #ops)
                },
            )
        }
    }

    let output_enum_doc_str = r" Possible outputs to request from the simulation     

    ## Example   

    ```json
    {{#include ../../../model/tests/box.spl:bedroom}}
    {{#include ../../../model/tests/box.spl:bedroom_output}}
    ```
    ";

    let output_enum_doc = sanitize_docs(output_enum_doc_str);

    let get_output_string_docs = " Produces the String that would be needed to ask `SIMPLE` to output the value of `SimulationStateElement`";
    Ok(quote!(

        impl #enum_name {
            #[doc=#get_output_string_docs]
            pub fn stringify(&self, model: &crate::Model)->String{
                match self {
                    #get_output_string
                    _ => todo!("Unsupported transformation between SimulationStateElement into Output --> {:?}", self)
                }


            }
        }

        #output_enum_doc
        #[derive(Clone, Debug, derive::ObjectIO, PartialEq,Eq, serde::Serialize, serde::Deserialize)]
        pub enum Output {

            #output_enum
        }
    ))
}
