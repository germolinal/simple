/*
MIT License
Copyright (c) 2021 Germán Molina
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

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, ItemFn};

/// An Attribute MAcro
#[proc_macro_attribute]
pub fn valid(title: TokenStream, item: TokenStream) -> TokenStream {
    let item2 = item.clone();
    let item_ast = parse_macro_input!(item2 as ItemFn);
    let item = proc_macro2::TokenStream::from(item);
    let title = title.to_string().replace("\"", "");
    let v = &item_ast.attrs;
    let docs = get_docs(v);
    let span = proc_macro2::Span::call_site();
    let function_name = find_name(item.clone());
    
    let result_type = get_output_type(&item_ast.sig.output);
    let returns_result = format!("{}", result_type).starts_with("Result");

    let ret_statement = if returns_result {
        quote!(Ok(Box::new(wrapper)))
    }else{
        quote!(Box::new(wrapper))
    };



    let wrapper_result_type = if returns_result {
        quote!(#result_type)
    }else{
        quote!(Result<#result_type, String>)
    };

    // Function is all right
    let item = if returns_result {
        quote!(#item)
    }else{
        // function needs to return a Result
        quote!( fn #function_name()->#wrapper_result_type {
            #item

            Ok(#function_name())
        } )
    };

    let output = quote_spanned!(span =>

        fn #function_name ()-> #result_type {

            #item

            let t : fn() -> #wrapper_result_type = #function_name;

            let wrapper = validate::ValidatorWrapper{
                title: #title .to_string(),
                description: #docs.into(),
                val: t
            };

            #ret_statement
        }
    );
    output.into()
}


/// Get the output type of a function based on its return type
fn get_output_type(ret: &syn::ReturnType) -> proc_macro2::TokenStream {
    match ret {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    }
}


fn get_docs(attrs: &[syn::Attribute]) -> String {
    let mut ret = String::new();

    for at in attrs {
        if let Some(segment) = at.path.segments.iter().next() {
            let segment_ident = format!("{}", segment.ident);
            if "doc" == segment_ident {
                let mut doc = format!("{}", at.tokens.clone());
                // Get rid of the annoying '=' and '"'
                doc.remove(0);
                doc.remove(1);
                doc.remove(doc.len() - 1);

                let doc = doc.replace("\\\\", "\\");
                let doc = doc.replace("\\\"", "\"");

                ret.push_str(&format!("{}\n", doc));
            }
        }
    }

    ret
}

/// This is a copy from [Rust-Criterion](https://github.com/bheisler/criterion.rs)
fn find_name(stream: proc_macro2::TokenStream) -> Ident {
    let mut iter = stream.into_iter();
    // while let Some(tok) = iter.next() {
    for tok in iter.by_ref() {
        if let TokenTree::Ident(ident) = tok {
            if ident == "fn" {
                break;
            }
        }
    }

    if let Some(TokenTree::Ident(name)) = iter.next() {
        name
    } else {
        panic!("Unable to find function name")
    }
}
