use crate::field::Field;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::fmt::Write as _; // import without risk of name clashing

#[derive(Clone)]
pub struct VariantData {
    pub ident: syn::Ident,
    #[allow(dead_code)]
    pub attributes: Vec<String>,
    pub fields: Vec<Field>,
    pub docs: String,
    // pub ty : syn::Type,
}

#[derive(Clone)]
pub enum Variant {
    Unit(VariantData),
    Unnamed(VariantData),
    Named(VariantData),
}

impl Variant {
    fn data(&self) -> VariantData {
        match self {
            Self::Unit(d) => d.clone(),
            Self::Unnamed(d) => d.clone(),
            Self::Named(d) => d.clone(),
        }
    }

    pub fn new(variant: syn::Variant) -> Self {
        let ident = variant.ident.clone();

        let attributes: Vec<String> = variant
            .attrs
            .iter()
            .map(|a| format!("{}", a.path().segments[0].ident))
            .collect();

        let docs = crate::docs::get_docs(&variant.attrs).expect("Could not generate docs");

        let mut data = VariantData {
            ident,
            attributes,
            fields: Vec::new(),
            docs,
        };
        match &variant.fields {
            syn::Fields::Unit => Self::Unit(data),
            syn::Fields::Unnamed(fields) => {
                data.fields = fields
                    .unnamed
                    .iter()
                    .map(|x| Field::new(x.clone()).expect("Could not create field"))
                    .collect();
                Self::Unnamed(data)
            }
            syn::Fields::Named(named_fields) => {
                data.fields = named_fields
                    .named
                    .clone()
                    .into_iter()
                    .filter_map(|f| {
                        // Skip State fields
                        let a = Field::new(f).expect("Could not create field");
                        match a {
                            Field::State(_) => None,
                            _ => Some(a),
                        }
                    })
                    .collect();

                Self::Named(data)
            }
        }
    }
}

pub struct EnumObject {
    pub ident: syn::Ident,
    pub variants: Vec<Variant>,
    docs: String,
    pub attributes: Vec<String>,
}

impl EnumObject {
    pub fn new(
        ident: syn::Ident,
        stru: syn::DataEnum,
        docs: String,
        attributes: Vec<String>,
    ) -> Self {
        let variants: Vec<Variant> = stru
            .variants
            .iter()
            .map(|x| Variant::new(x.clone()))
            .collect();

        Self {
            variants,
            ident,
            docs,
            attributes,
        }
    }

    pub fn gen_display(&self) -> TokenStream2 {
        let ret = quote!(
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let j = serde_json::to_string_pretty(&self).unwrap();
                write!(f, "{}\n\n", j)
            }
        );

        ret
    }

    pub fn gen_docs(&self) -> Result<String, String> {
        let mut ret = String::new();

        // // Title
        writeln!(ret, "# {}\n", &self.ident).map_err(|e| e.to_string())?;

        // Written doc
        writeln!(ret, "{}\n\n ## Supported Variants\n", &self.docs).map_err(|e| e.to_string())?;

        // Document variants
        for variant in self.variants.iter() {
            writeln!(ret, "###  {}\n", &variant.data().ident).map_err(|e| e.to_string())?;

            writeln!(ret, "{}\n\n", &variant.data().docs).map_err(|e| e.to_string())?;
            match variant {
                Variant::Unnamed(s) => {
                    let v_ident = &s.ident;
                    write!(ret, "```rs\n{} {{\n\t{} : ", &self.ident, v_ident)
                        .map_err(|e| e.to_string())?;
                    let mut i = 0;
                    for field in s.fields.iter() {
                        if let Field::State(_) = field {
                            continue;
                        }
                        if i != 0 {
                            ret += ","
                        }

                        ret += &field.get_documentation_type()?.to_string();
                        i += 1;
                    }
                    ret += "\n}\n```\n\n";
                }
                Variant::Named(s) => {
                    let v_ident = &s.ident;
                    write!(ret, "##### Full Specification\n```json\n{} {{\n\ttype : \"{}\", // this should not change\n", &self.ident, v_ident).unwrap();
                    let mut i = 0;

                    for field in s.fields.iter() {
                        if let Field::State(_) = field {
                            continue;
                        }

                        let s_ident = &field.data().ident.ok_or("No identity")?;
                        write!(ret, "\t{} : ", s_ident).map_err(|e| e.to_string())?;

                        ret += &field.get_documentation_type()?.to_string();
                        if i != 0 {
                            ret += ",\n"
                        } else {
                            ret += "\n"
                        }
                        i += 1;
                    }
                    ret += "}\n```\n\n";
                }
                Variant::Unit(s) => {
                    let v_ident = &s.ident;
                    if self.attributes.contains(&"inline_enum".to_string()) {
                        writeln!(
                            ret,
                            "##### Full Specification\n```json\n\"{}\"\n```\n",
                            v_ident
                        )
                        .map_err(|e| e.to_string())?;
                    } else {
                        writeln!(
                            ret,
                            "##### Full Specification\n```json\n{{\n\ttype: \"{}\"\n}}\n```\n\n",
                            v_ident
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        // Api access.
        let object_name_str = format!("{}", &self.ident);
        if crate::object_has_api(object_name_str.clone()) {
            let name_str_lower = object_name_str.to_lowercase();
            writeln!(ret, "\n\n## API Access\n\n```rs\n// by name\n let my_{} = {}(string);\n// by index\nlet my_{} = {}(int);```", name_str_lower, name_str_lower, name_str_lower, name_str_lower).unwrap();
        }

        Ok(ret)
    }

    pub fn gen_group_behaviour(&self) -> TokenStream2 {
        let object_name = &self.ident;
        let object_docs = &self.docs;
        // let err_colon_colon = format!("Expecting '::' after group name '{}'", object_name);

        let mut from_bytes = quote!();
        let mut name_fn = quote!();

        let mut doc_string = format!(
            "# {}\n\n{}\n\n ## Supported Types\n\n",
            object_name, object_docs
        );

        for variant in self.variants.iter() {
            let ident = variant.data().ident;
            let ident_str = format!("{}", ident);
            let variant_docs = &variant.data().docs;

            name_fn = quote!(
                #name_fn

                #object_name::#ident(o) => {
                    &o.name
                }
            );

            // Extend from_bytes() match statement
            from_bytes = quote!(
                #from_bytes

                #ident_str => {
                    // println!("Variant is {}", #ident);
                    let ret = #ident::from_bytes(scanner.line, slice, model)?;
                    Ok(Self::#ident(std::sync::Arc::new(ret)))
                }
            );

            // Extend docs
            writeln!(doc_string, "* **{}**: {}", ident, variant_docs).unwrap();
        }
        // Api access.
        let object_name_str = format!("{}", object_name);
        if crate::object_has_api(object_name_str.clone()) {
            let name_str_lower = object_name_str.to_lowercase();
            writeln!(doc_string, "\n\n## API Access\n\n```rs\n// by name\nlet my_{} = {}(string);\n// by index\nlet my_{} = {}(int);\n```", name_str_lower, name_str_lower, name_str_lower, name_str_lower).unwrap();
        }

        name_fn = quote!(
            /// Borrows the name
            pub fn name(&self)->&String{
                match self{
                    #name_fn
                }
            }
        );

        let print_doc = quote!(

            /// Prints the docs for this object
            #[cfg(debug_assertions)]
            pub fn print_doc(dir: &str, summary: &mut String)->std::io::Result<()>{
                let doc = #doc_string.as_bytes() ;

                let filename = format!("auto-{}.md", #object_name_str).to_lowercase();
                let full_filename = format!("{}/{}", dir, filename);
                #[allow(clippy::format_push_string)] // this is really not relevant... it happens only during testing, not at runtime.
                summary.push_str(&format!("- [{}](./{})\n",#object_name_str, filename));

                std::fs::write(&full_filename, doc)?;
                Ok(())

            }
        );

        quote!(
            impl #object_name {

                // #from_bytes

                #name_fn

                #print_doc
            }
        )
    }

    pub fn gen_group_api(&self) -> TokenStream2 {
        let object_name = self.ident.clone();
        let name_str = format!("{}", &object_name);
        let name_str_lower = name_str.to_lowercase();

        let location_str = crate::object_location(name_str).unwrap_or_else(|| {
            panic!(
                "Trying to gen a Group API of '{}', which is not registered in Model",
                &object_name
            )
        });
        let location = syn::Ident::new(location_str, proc_macro2::Span::call_site());
        let not_found_err = format!("Could not find {} '{{}}'", &object_name);
        let negative_index_err = format!(
            "Impossible to get {} using a negative index ({{}} was given)",
            &object_name
        );
        let out_of_bounds_err = format!(
            "Trying to access {} number {{}}... but the last index is {{}}",
            &object_name
        );

        let mut name_match_statement = quote!();
        let mut index_match_statement = quote!();

        for v in self.variants.iter() {
            let v_ident = v.data().ident.clone();

            name_match_statement = quote!(
                #name_match_statement

                #object_name::#v_ident(s)=>{
                    if s.name == name {
                        let d = rhai::Dynamic::from(std::sync::Arc::clone(s));
                        return Ok(d)
                    }
                }
            );

            index_match_statement = quote!(
                #index_match_statement

                #object_name::#v_ident(s)=>{
                    let d = rhai::Dynamic::from(std::sync::Arc::clone(s));
                    return Ok(d)
                }

            )
        }

        let api_doc_string = format!(" Registers the Rhai API for `{}`", object_name);
        quote!(
            impl #object_name {
                #[doc = #api_doc_string]
                pub fn register_api(engine : &mut rhai::Engine, model: &std::sync::Arc<Model>, state: &std::sync::Arc<std::cell::RefCell<crate::SimulationState>>, research_mode: bool){

                    // By name
                    let new_mod = std::sync::Arc::clone(model);
                    engine.register_fn(#name_str_lower, move |name: &str | -> Result<_, Box<rhai::EvalAltResult>> {

                        for s in new_mod.#location.iter(){
                            match s {
                                #name_match_statement
                            }
                        }
                        return Err(format!(#not_found_err, name).into());
                    });

                    // By index
                    let new_mod = std::sync::Arc::clone(model);
                    engine.register_fn(#name_str_lower, move |index: rhai::INT|   -> Result<_, Box<rhai::EvalAltResult>> {

                        let len = new_mod.#location.len();
                        if index < 0 {
                            return Err(format!(#negative_index_err, index).into())
                        }
                        if index >= len as i64 {
                            return Err(format!(#out_of_bounds_err, index, len - 1).into());
                        }
                        match &new_mod.#location[index as usize]{
                            #index_match_statement
                        }

                    });

                }
            }
        )
    }
}
