use crate::common_path::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Meta, MetaList, MetaNameValue};

pub const STATE_ELEMENT_TYPE: &str = "StateElementField";

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

impl Attribute {
    pub fn new(a: &syn::Attribute) -> Self {
        if let syn::AttrStyle::Inner(_) = &a.style {
            panic!("Expecing Outer style attribute")
        }
        let name = a.path().segments[0].ident.clone();
        let mut value: Option<String> = None;

        if name != "doc" {
            value = match &a.meta {
                Meta::Path(_) => {
                    // dbg!(&name, "Path!");
                    // p.segments.clone()
                    None
                }
                Meta::NameValue(MetaNameValue { .. }) => {
                    unreachable!(
                        "I do not think we support attributes of this kind... please report this"
                    );
                }
                Meta::List(MetaList {
                    path: _, tokens, ..
                }) => Some(format!("{}", tokens)),
            };
        }

        Self {
            name: format!("{}", name),
            value,
        }
    }
}

#[derive(Clone)]
pub struct FieldData {
    pub ident: Option<syn::Ident>,
    pub attributes: Vec<Attribute>,
    pub child: Option<Box<Field>>,
    pub ty: syn::Type,
    pub docs: Option<String>,
    pub api_alias: Option<String>,
}

#[derive(Clone)]
pub enum Field {
    Float(FieldData),
    Int(FieldData),
    Bool(FieldData),
    String(FieldData),

    Option(FieldData),
    Vec(FieldData),
    Rc(FieldData),

    Object(FieldData),
    State(FieldData),
}

impl Field {
    /// Creates a new Field object, with no ident and no attributes.
    ///
    /// This method is meant to be used for nested typs. E.g. the `usize` in `Vec<usize>`
    pub fn from_type(ty: &syn::Type) -> Result<Self, String> {
        let mut data = FieldData {
            ident: None,
            attributes: Vec::new(),
            child: None,
            ty: ty.clone(),
            docs: None,      // This is nested... the Docs should be in the parent
            api_alias: None, // This is nested... the Docs should be in the parent
        };

        if let syn::Type::Path(p) = ty {
            if path_is_float(&p.path)? {
                Ok(Self::Float(data))
            } else if path_is_int(&p.path)? {
                Ok(Self::Int(data))
            } else if path_is_bool(&p.path)? {
                Ok(Self::Bool(data))
            } else if path_is_string(&p.path)? {
                Ok(Self::String(data))
            } else if path_is_option(&p.path)? {
                let ty = extract_type_from_path(&p.path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Option(data))
            } else if path_is_vec(&p.path)? {
                let ty = extract_type_from_path(&p.path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Vec(data))
            } else if path_is_rc(&p.path)? {
                let ty = extract_type_from_path(&p.path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Rc(data))
            } else {
                let ty_str = path_to_string(&p.path)?;

                if ty_str == STATE_ELEMENT_TYPE {
                    return Ok(Self::State(data));
                }
                // Therea re object—e.g., enums—that are stored in other object, not in models
                Ok(Self::Object(data))
            }
        } else {
            panic!("TODO: Error handling 3")
        }
    }

    /// Creates a new Field object based on the tokens in an actual Struct.    
    pub fn new(field: syn::Field) -> Result<Self, String> {
        let ident = field.ident.clone();

        if let syn::Type::Path(t) = &field.ty {
            // field.attrs.iter().for_each(|a|{
            //     dbg!(a);
            // });
            let attributes: Vec<Attribute> = field.attrs.iter().map(Attribute::new).collect();

            let mut api_alias: Option<String> = None;
            for a in attributes.iter() {
                if a.name == "physical" || a.name == "operational" {
                    api_alias = a.value.clone();
                    break;
                }
            }

            let path = t.path.clone();
            let ty = field.ty.clone();
            let docs = Some(crate::docs::get_docs(&field.attrs)?);

            let mut data = FieldData {
                ident,
                attributes,
                ty,
                docs,
                api_alias,
                child: None,
            };

            if path_is_float(&path)? {
                Ok(Self::Float(data))
            } else if path_is_int(&path)? {
                Ok(Self::Int(data))
            } else if path_is_bool(&path)? {
                Ok(Self::Bool(data))
            } else if path_is_string(&path)? {
                Ok(Self::String(data))
            } else if path_is_option(&path)? {
                let ty = extract_type_from_path(&path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Option(data))
            } else if path_is_vec(&path)? {
                let ty = extract_type_from_path(&path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Vec(data))
            } else if path_is_rc(&path)? {
                let ty = extract_type_from_path(&path)?;
                let aux = Self::from_type(&ty)?;
                data.child = Some(Box::new(aux));
                Ok(Self::Rc(data))
            } else {
                let ty_str = path_to_string(&path)?;
                if ty_str == STATE_ELEMENT_TYPE {
                    return Ok(Self::State(data));
                }
                // Therea re object—e.g., enums—that are stored in other object, not in models
                Ok(Self::Object(data))
            }
        } else {
            panic!(
                "Unhandled syn::Type '{:?}' when Field::new()",
                quote!(field.ty)
            )
        }
    }

    pub fn data(&self) -> FieldData {
        match self {
            Self::Float(d)
            | Self::Int(d)
            | Self::Bool(d)
            | Self::String(d)
            | Self::Vec(d)
            | Self::Rc(d)
            | Self::Option(d)
            | Self::Object(d)
            | Self::State(d) => d.clone(),
        }
    }

    pub fn api_name(&self) -> Result<String, String> {
        let data = self.data();
        let ret = match data.api_alias {
            Some(a) => (a[1..a.len() - 1]).to_string(),
            None => data.ident.clone().ok_or("No ident found")?.to_string(),
        };
        Ok(ret)
    }

    pub fn api_getter(&self, object_name: &syn::Ident) -> Result<TokenStream2, String> {
        let fieldname = &self.data().ident.ok_or("Did not find any ident")?;

        let api_fieldname = self.api_name()?;

        let value_not_available_err = format!(
            "{} called '{{}}' has not been assigned a value for property '{}'",
            object_name, api_fieldname
        );
        Ok(quote!(
            // Getter by name
            let new_mod = std::sync::Arc::clone(model);
            let new_state = std::sync::Arc::clone(state);
            engine.register_get_result(#api_fieldname, move |this: &mut std::sync::Arc<#object_name>| {
                let state_ptr = & *new_state.borrow();
                match this.#fieldname(state_ptr){
                    Some(v)=> {return Ok(v)},
                    None => {return Err(format!(#value_not_available_err, this.name).into());}
                }
            });
        ))
    }

    pub fn api_setter(&self, object_name: &syn::Ident) -> Result<TokenStream2, String> {
        let data = self.data();
        let fieldname = &data.ident.clone().ok_or("no ident found for data")?;
        let api_fieldname = match data.api_alias {
            Some(a) => (a[1..a.len() - 1]).to_string(),
            None => fieldname.to_string(),
        };

        let rust_fn = syn::Ident::new(
            &format!("set_{}", fieldname),
            proc_macro2::Span::call_site(),
        );
        let index_ident = format!("{}_index", fieldname);
        let index_ident = syn::Ident::new(&index_ident, fieldname.span());
        let value_not_available_err = format!(
            "Property '{}' has not been initialized for {} called '{{}}' ",
            api_fieldname, object_name
        );
        let ret = quote!(
            // Setter by name
            let new_mod = std::sync::Arc::clone(model);
            let new_state = std::sync::Arc::clone(state);
            engine.register_set(#api_fieldname, move |this: &mut std::sync::Arc<#object_name>, v: crate::Float | -> Result<_, Box<rhai::EvalAltResult>> {
                match this.#index_ident(){
                    Some(_)=>{
                        let state_ptr = &mut *new_state.borrow_mut();
                        this.#rust_fn(state_ptr, v)?;
                    },
                    None => {
                        return Err(format!(#value_not_available_err, this.name).into());
                    }
                };
                Ok(())

            });

            let new_mod = std::sync::Arc::clone(model);
            let new_state = std::sync::Arc::clone(state);
            engine.register_set(#api_fieldname, move |this: &mut std::sync::Arc<#object_name>, v: rhai::INT | -> Result<_, Box<rhai::EvalAltResult>> {
                match this.#index_ident(){
                    Some(_)=>{
                        let state_ptr = &mut *new_state.borrow_mut();
                        this.#rust_fn(state_ptr, v as crate::Float)?;
                    },
                    None => {
                        return Err(format!(#value_not_available_err, this.name).into());
                    }
                };
                Ok(())

            });
        );

        Ok(ret)
    }

    pub fn get_documentation_type(&self) -> Result<String, String> {
        let r = match self {
            Field::Float(_) => "number".to_string(),
            Field::Int(_) => "int".to_string(),
            Field::Bool(_d) => "boolean".to_string(),
            Field::String(_d) => "string".to_string(),

            Field::Vec(d) => {
                let child_type = d
                    .child
                    .clone()
                    .ok_or("no child?")?
                    .get_documentation_type()?;
                format!("[{}, ...]", child_type)
            }
            Field::Rc(d) => d
                .child
                .clone()
                .ok_or("no child found")?
                .get_documentation_type()?,
            Field::Option(d) => {
                let child_type = d
                    .child
                    .clone()
                    .ok_or("no child?")?
                    .get_documentation_type()?;
                format!("{}, // optional", child_type)
            }
            Field::Object(d) => {
                if let syn::Type::Path(t) = &d.ty {
                    path_to_string(&t.path)?
                } else {
                    unreachable!("Weird object when getting docs")
                }
            }
            Field::State(_d) => {
                unreachable!("Trying to doc-type of State field")
            }
        };
        Ok(r)
    }

    pub fn get_documentation(&self) -> Result<String, String> {
        let f_ident = self.data().ident.ok_or("No ident")?;

        let d = match self {
            Field::Float(_)
            | Field::Int(_)
            | Field::Bool(_)
            | Field::String(_)
            | Field::Vec(_)
            | Field::Rc(_)
            | Field::Object(_)
            | Field::Option(_) => {
                format!("{} : {}", f_ident, self.get_documentation_type()?)
            }
            Field::State(_d) => {
                panic!("Trying to get verification for State field")
            }
        };
        Ok(d)
    }
}
