use std::fmt::Write as _;

use syn::{Lit, Meta, MetaNameValue}; // import without risk of name clashing

pub fn get_docs(attrs: &[syn::Attribute]) -> Result<String, String> {
    // let mut ret = String::new();
    let mut ret = "".to_string();

    for at in attrs {
        if let Some(segment) = at.path().segments.iter().next() {
            let segment_ident = format!("{}", segment.ident);
            if "doc" == segment_ident {
                let exp = match &at.meta {
                    Meta::NameValue(MetaNameValue { path: _, value, .. }) => {
                        if let syn::Expr::Lit(v) = value {
                            if let Lit::Str(token) = &v.lit {
                                token.token().clone()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(
                        "Docs should not be anything than NameValue... please report this"
                    ),
                };

                // extract the content
                let doc = exp.to_string();
                let mut c = doc.chars();
                c.next();
                c.next();
                c.next_back();
                let doc = c.as_str().to_string();

                writeln!(ret, "{}", doc).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(ret)
}
