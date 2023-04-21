use crate::common_path::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

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
        let name = a.path.segments[0].ident.clone();
        let mut value: Option<String> = None;
        a.tokens.clone().into_iter().for_each(|token| {
            if let proc_macro2::TokenTree::Group(g) = token {
                g.stream().into_iter().for_each(|literal| {
                    if let proc_macro2::TokenTree::Literal(lit) = literal {
                        value = Some(format!("{}", lit));
                    }
                })
            }
        });
        // return
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
    pub fn from_type(ty: &syn::Type) -> Self {
        let mut data = FieldData {
            ident: None,
            attributes: Vec::new(),
            child: None,
            ty: ty.clone(),
            docs: None,      // This is nested... the Docs should be in the parent
            api_alias: None, // This is nested... the Docs should be in the parent
        };

        if let syn::Type::Path(p) = ty {
            if path_is_float(&p.path) {
                Self::Float(data)
            } else if path_is_int(&p.path) {
                Self::Int(data)
            } else if path_is_bool(&p.path) {
                Self::Bool(data)
            } else if path_is_string(&p.path) {
                Self::String(data)
            } else if path_is_option(&p.path) {
                let ty = extract_type_from_path(&p.path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Option(data)
            } else if path_is_vec(&p.path) {
                let ty = extract_type_from_path(&p.path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Vec(data)
            } else if path_is_rc(&p.path) {
                let ty = extract_type_from_path(&p.path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Rc(data)
            } else {
                let ty_str = path_to_string(&p.path);

                if ty_str == STATE_ELEMENT_TYPE {
                    return Self::State(data);
                }
                // Therea re object—e.g., enums—that are stored in other object, not in models
                Self::Object(data)
            }
        } else {
            panic!("TODO: Error handling 3")
        }
    }

    /// Creates a new Field object based on the tokens in an actual Struct.    
    pub fn new(field: syn::Field) -> Self {
        let ident = field.ident.clone();

        if let syn::Type::Path(t) = &field.ty {
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
            let docs = Some(crate::docs::get_docs(&field.attrs));

            let mut data = FieldData {
                ident,
                attributes,
                ty,
                docs,
                api_alias,
                child: None,
            };

            if path_is_float(&path) {
                Self::Float(data)
            } else if path_is_int(&path) {
                Self::Int(data)
            } else if path_is_bool(&path) {
                Self::Bool(data)
            } else if path_is_string(&path) {
                Self::String(data)
            } else if path_is_option(&path) {
                let ty = extract_type_from_path(&path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Option(data)
            } else if path_is_vec(&path) {
                let ty = extract_type_from_path(&path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Vec(data)
            } else if path_is_rc(&path) {
                let ty = extract_type_from_path(&path).unwrap();
                data.child = Some(Box::new(Self::from_type(&ty)));
                Self::Rc(data)
            } else {
                let ty_str = path_to_string(&path);
                if ty_str == STATE_ELEMENT_TYPE {
                    return Self::State(data);
                }
                // Therea re object—e.g., enums—that are stored in other object, not in models
                Self::Object(data)
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

    pub fn api_name(&self) -> String {
        let data = self.data();
        match data.api_alias {
            Some(a) => (a[1..a.len() - 1]).to_string(),
            None => data.ident.clone().unwrap().to_string(),
        }
    }

    pub fn api_getter(&self, object_name: &syn::Ident) -> TokenStream2 {
        let fieldname = &self.data().ident.unwrap();

        let api_fieldname = self.api_name();

        let value_not_available_err = format!(
            "{} called '{{}}' has not been assigned a value for property '{}'",
            object_name, api_fieldname
        );
        quote!(
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
        )
    }

    pub fn api_setter(&self, object_name: &syn::Ident) -> TokenStream2 {
        let data = self.data();
        let fieldname = &data.ident.clone().unwrap();
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
        quote!(
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
        )
    }

    pub fn get_documentation_type(&self) -> String {
        match self {
            Field::Float(_) => "number".to_string(),
            Field::Int(_) => "int".to_string(),
            Field::Bool(_d) => "boolean".to_string(),
            Field::String(_d) => "string".to_string(),

            Field::Vec(d) => {
                let child_type = d.child.clone().unwrap().get_documentation_type();
                format!("[{}, ...]", child_type)
            }
            Field::Rc(d) => d.child.clone().unwrap().get_documentation_type(),
            Field::Option(d) => {
                let child_type = d.child.clone().unwrap().get_documentation_type();
                format!("{}, // optional", child_type)
            }
            Field::Object(d) => {
                if let syn::Type::Path(t) = &d.ty {
                    path_to_string(&t.path)
                } else {
                    panic!("Weird object when getting docs")
                }
            }
            Field::State(_d) => {
                panic!("Trying to doc-type of State field")
            }
        }
    }

    pub fn get_documentation(&self) -> String {
        let f_ident = self.data().ident.unwrap();

        match self {
            Field::Float(_)
            | Field::Int(_)
            | Field::Bool(_)
            | Field::String(_)
            | Field::Vec(_)
            | Field::Rc(_)
            | Field::Object(_)
            | Field::Option(_) => {
                format!("{} : {}", f_ident, self.get_documentation_type())
            }
            Field::State(_d) => {
                panic!("Trying to get verification for State field")
            }
        }
    }
}
