// use proc_macro::TokenStream;
use crate::field::Field;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::fmt::Write as _; // import without risk of name clashing

pub struct StructObject {
    pub ident: syn::Ident,
    pub fields: Vec<Field>,
    docs: String,
    // optional_fields: Vec<Field>,
    // state_fields: Vec<Field>,
}

impl StructObject {
    pub fn new(ident: syn::Ident, stru: syn::DataStruct, docs: String) -> Self {
        let fields: Vec<Field> = Self::get_object_fields(&stru)
            .into_iter()
            .filter_map(|x| Field::new(x).ok())
            .collect();

        StructObject {
            ident,
            fields,
            docs,
        }
    }

    pub fn has_name(&self) -> bool {
        for f in self.fields.iter() {
            if let crate::field::Field::String(sf) = f {
                if let Some(i) = &sf.ident {
                    let ident_str = format!("{}", i);
                    if ident_str == "name" {
                        return true;
                    }
                }
            }
        }
        false
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

        // Title
        writeln!(ret, "# {}\n", self.ident).map_err(|e| e.to_string())?;

        // Written doc
        let docs = &self.docs;
        writeln!(ret, "{}\n\n ## Full Specification\n", docs).map_err(|e| e.to_string())?;

        // A codeblock with the all the fields
        ret += "```json\n"; // Open template
        writeln!(ret, "{} {{", self.ident).map_err(|e| e.to_string())?;

        for field in self.fields.iter() {
            let f_ident = field.data().ident.clone().ok_or("No identity")?;
            if f_ident == "index" {
                continue;
            }
            if let Field::State(_) = field {
                continue;
            }
            writeln!(ret, "   {},", field.get_documentation()?).map_err(|e| e.to_string())?;
        }
        ret += "}\n```\n\n";

        // Documentations for fields
        for field in self.fields.iter() {
            let f_ident = field.data().ident.clone().ok_or("No identity")?;
            if f_ident == "index" {
                continue;
            }
            if let Field::State(_) = field {
                continue;
            }

            write!(ret, "\n\n#### {}", f_ident).map_err(|e| e.to_string())?;
            if let Field::Option(_) = field {
                ret += " (*optional*)";
            }
            ret += "\n\n";

            let errmsg = format!("Field '{}' has no docs", f_ident);
            writeln!(ret, "{}\n", field.data().docs.expect(&errmsg)).map_err(|e| e.to_string())?;
        }
        ret += "\n\n";

        // Api access.
        let object_name_str = format!("{}", self.ident);
        if crate::object_has_api(object_name_str.clone()) {
            let name_str_lower = object_name_str.to_lowercase();
            write!(ret, "\n\n## API Access\n\n```rs\n// by name\nlet my_{} = {}(string);\n// by index\nlet my_{} = {}(int);\n```", name_str_lower, name_str_lower, name_str_lower, name_str_lower).map_err(|e| e.to_string())?;
        }

        Ok(ret)
    }

    pub fn gen_new(&self) -> Result<TokenStream2, String> {
        let req_field_names = self.collect_required_fields();
        let new_docstring = format!(" Creates a new [`{}`]", self.ident);
        let mut content = quote!();

        // Initialize all values

        let mut any_strings = false;
        for f in self.fields.iter() {
            let fname = f.data().ident.clone().ok_or("No identity")?;

            match f {
                Field::State(_d) => {
                    // states are empty
                    content = quote!(
                        #content
                        #fname : std::sync::Arc::new(std::sync::Mutex::new(None)),
                    )
                }
                Field::Option(_) => {
                    // optionals are none
                    content = quote!(
                        #content
                        #fname : None,
                    )
                }
                Field::Vec(_) => {
                    // vecs are empty
                    content = quote!(
                        #content
                        #fname : Vec::new(),
                    )
                }
                Field::String(_) => {
                    any_strings = true;
                    // Strings are passed as generics
                    content = quote!(
                        #content
                        #fname : #fname.into(),
                    )
                }
                _ => {
                    content = quote!(
                        #content
                        #fname,
                    )
                }
            }
        }

        if any_strings {
            Ok(quote!(
                #[doc = #new_docstring]
                ///
                /// All the required fields are asked by the constructor. The Optional
                /// fields default to `None`, and the `Vec` fields default to emptt.
                pub fn new<S: Into<String>>(#req_field_names)->Self{
                    Self{
                        #content
                    }
                }
            ))
        } else {
            Ok(quote!(
                #[doc = #new_docstring]
                ///
                /// All the required fields are asked by the constructor. The Optional
                /// fields default to `None`, and the `Vec` fields default to emptt.
                pub fn new(#req_field_names)->Self{
                    Self{
                        #content
                    }
                }
            ))
        }
    }

    fn collect_required_fields(&self) -> TokenStream2 {
        let mut req_field_names = quote!();
        self.fields.iter().enumerate().for_each(|(i, f)| match f {
            Field::Option(_) | Field::State(_) | Field::Vec(_) => {}
            Field::String(_) => {
                let data = f.data();
                let f_ident = data.ident;
                if i == 0 {
                    req_field_names = quote!(#f_ident: S);
                } else {
                    req_field_names = quote!(#req_field_names, #f_ident : S);
                }
            }
            _ => {
                let data = f.data();
                let f_ident = data.ident;
                let f_type = data.ty;
                if i == 0 {
                    req_field_names = quote!(#f_ident: #f_type);
                } else {
                    req_field_names = quote!(#req_field_names, #f_ident : #f_type);
                }
            }
        });
        req_field_names
    }

    fn get_object_fields(
        stru: &syn::DataStruct,
    ) -> syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
        if let syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        } = stru
        {
            named.clone()
        } else {
            panic!(
                "Unhandled object when get_object_fields... {:?}",
                quote!(stru)
            );
        }
    }

    pub fn gen_state_getters_setters(&self) -> Result<TokenStream2, String> {
        let mut gets: TokenStream2 = quote!();
        let mut sets: TokenStream2 = quote!();

        for f in self.fields.iter() {
            // name of the field
            let f_ident = f.data().ident.clone().ok_or("No identity")?;
            match f {
                Field::State(_d) => {
                    /* SET THE INDEX OF THE OBJECT */
                    // name of the 'set_index_' method
                    let set_ident = format!("set_{}_index", f_ident);
                    let set_ident = syn::Ident::new(&set_ident, f_ident.span());
                    // doc of the 'set_index_' method
                    let sets_index_doc_string = format!(" Sets the index of the [`SimulationStateElement`] representing the `{}` within the [`SimulationState`]. Returns an error if the object already has an index assigned for this field.", f_ident);

                    let already_there_err = if self.has_name() {
                        let err_msg = format!(
                            "Field '{}' in {} called '{{}}' has already been asigned",
                            self.ident, f_ident
                        );
                        quote!(format!(#err_msg, self.name()))
                    } else {
                        let err_msg = format!(
                            "Field '{}' in {} has already been asigned",
                            self.ident, f_ident
                        );
                        quote!(format!(#err_msg))
                    };

                    let cant_lock_err = if self.has_name() {
                        let err_msg = format!(
                            "Field '{}' in {} called '{{}}' cannot be locked for reading and/or writing.",
                            self.ident, f_ident
                        );
                        quote!(#err_msg, self.name())
                    } else {
                        let err_msg = format!(
                            "Field '{}' in {} cannot be locked for reading and/or writing.",
                            self.ident, f_ident
                        );
                        quote!(#err_msg)
                    };

                    sets = quote!(
                        #sets

                        #[doc = #sets_index_doc_string]
                        pub fn #set_ident(&self, i: usize)->Result<(), String>{
                            // todo!();
                            if let Ok(mut guard) = self.#f_ident.lock(){
                                if guard.is_some() {
                                    return Err(#already_there_err);
                                }
                                *guard = Some(i);
                                return Ok(())
                            }else{
                                return Err(format!(#cant_lock_err))

                            }
                        }
                    );

                    /* SET THE STATE */
                    let set_ident = format!("set_{}", f_ident);
                    let set_ident = syn::Ident::new(&set_ident, f_ident.span());
                    // doc of the 'set_' method
                    let sets_doc_string = format!(" Changes the value of the [`SimulationStateElement`] associated with the `{}` within the [`SimulationState`] .", f_ident);
                    let err_msg = format!(
                        " Impossible to change the state of object because `{}` has no value",
                        f_ident
                    );
                    sets = quote!(
                        #sets

                        #[doc = #sets_doc_string]
                        pub fn #set_ident(&self, state: &mut crate::simulation_state::SimulationState, v : crate::Float) -> Result<(), String>{
                            if let Ok(guard) = self.#f_ident.lock(){
                                if let Some(i) = *guard {
                                    state[i] = v;
                                    return Ok(());
                                }else{
                                    return Err(format!("{}", #err_msg))
                                }
                            }else{
                                return Err(format!(#cant_lock_err))
                            }
                        }
                    );

                    /* GET THE INDEX OF THE OBJECT */
                    // name of the 'get_index_' method
                    let get_index_ident = format!("{}_index", f_ident);
                    let get_index_ident = syn::Ident::new(&get_index_ident, f_ident.span());

                    // doc of the 'get_index_' method
                    let get_index_doc_string = format!(" Gets the index of the [`SimulationStateElement`] representing the `{}` within the [`SimulationState`].", f_ident);

                    gets = quote!(
                        #gets

                        #[doc = #get_index_doc_string]
                        pub fn #get_index_ident(&self) -> Option<usize> {
                            if let Ok(guard) = self.#f_ident.lock(){
                                *guard
                            }else{
                                panic!(#cant_lock_err)
                            }

                        }
                    );

                    /* GET THE STATE OF THE OBJECT */

                    // doc of the 'get_index_' method
                    let get_doc_string = format!(" Gets the value of the [`SimulationStateElement`] representing the `{}` within the [`SimulationState`].", f_ident);
                    gets = quote!(
                        #gets

                        #[doc = #get_doc_string]
                        pub fn #f_ident(&self, state: &crate::simulation_state::SimulationState) -> Option<crate::Float> {

                            if let Ok(guard) = self.#f_ident.lock(){
                                if let Some(i) = *guard {
                                    return Some(state[i]);
                                }else{
                                    return None
                                }
                            }else{
                                panic!(#cant_lock_err)
                            }
                        }
                    );
                }
                Field::Option(d) => {
                    let f_ident_str = format!("{}", f_ident);
                    // Type T inside the Option<T>
                    // let ty = extract_type_from_option(&f.ty).expect("When bulding build_optional_get_set() 0");
                    let ty = d.child.clone().unwrap().data().ty;
                    let mut is_string = false;
                    let mut is_copy = false;
                    if let Some(b) = &d.child {
                        match **b {
                            Field::String(_) => is_string = true,
                            Field::Int(_) | Field::Float(_) | Field::Bool(_) => is_copy = true,

                            _ => {}
                        }
                    }
                    // let ty = syn::Ident::new("f32", proc_macro2::Span::call_site());
                    // ident.unwrap();

                    // Name of the 'set_' method
                    let set_ident = format!("set_{}", f_ident);
                    let set_ident = syn::Ident::new(&set_ident, f_ident.span());
                    // Doc for the 'set_' method
                    let sets_doc_string = format!(" Sets the `{}` field.", f_ident);

                    if is_string {
                        sets = quote!(
                            #sets

                            #[doc = #sets_doc_string]
                            pub fn #set_ident<S: Into<String>>(&mut self, v: S)->&mut Self{
                                self.#f_ident = Some(v.into());
                                self
                            }
                        );
                    } else {
                        sets = quote!(
                            #sets

                            #[doc = #sets_doc_string]
                            pub fn #set_ident(&mut self, v: #ty)->&mut Self{
                                self.#f_ident = Some(v);
                                self
                            }
                        );
                    }

                    // Name of the 'set_' method
                    let get_or_ident = format!("{}_or", f_ident);
                    let get_or_ident = syn::Ident::new(&get_or_ident, f_ident.span());
                    let gets_or_doc_string = format!(" Gets the content of `{}` field or a default value given if the field is `None`. If no default is logical, use `self.{}()` instead, which returns an error", f_ident, f_ident);
                    // let gets_or_warning = if self.has_name() {
                    //     quote!(
                    //         crate::error_msgs::print_warning(module_name,
                    //             format!("{} called '{}' has not been assigned any value for field '{}'... assuming a value of {}", self.object_type(), self.name, #f_ident_str, default)
                    //         );
                    //     )
                    // } else {
                    //     quote!(
                    //         crate::error_msgs::print_warning(module_name,
                    //             format!("{} has not been assigned any value for field '{}'... assuming a value of {}", self.object_type(), #f_ident_str,  default)
                    //         );
                    //     )
                    // };

                    if is_copy {
                        gets = quote!(
                            #gets

                            #[doc = #gets_or_doc_string]
                            pub fn #get_or_ident<T: std::fmt::Display>(&self, module_name: T, default: #ty) -> #ty {
                                match &self.#f_ident {
                                    Some(v) => *v,
                                    None => {
                                        // #gets_or_warning
                                        default
                                    },
                                }
                            }

                        )
                    }

                    // the ident of this method is the name of the field itself
                    let gets_doc_string = format!(" Gets the `{}` field. Returns a `Result` because this field is optional and thus it might not be there.", f_ident);
                    let gets_err = if self.has_name() {
                        quote!(
                            format!("{} called '{}' has not been assigned any value for field '{}'", self.object_type(), self.name, #f_ident_str)
                        )
                    } else {
                        quote!(
                            format!("{} has not been assigned any value for field '{}'", self.object_type(),  #f_ident_str)
                        )
                    };
                    gets = quote!(
                        #gets



                        #[doc = #gets_doc_string]
                        pub fn #f_ident(&self) -> Result<&#ty, String> {
                            match &self.#f_ident {
                                Some(v) => Ok(v),
                                None => Err(#gets_err),
                            }
                        }
                    );
                }
                _ => { /* Do nothing */ }
            } // End of match
        } // end of fields.iter()

        Ok(quote!(
            #gets

            #sets
        ))
    }

    fn get_api_getters_setters_docs(&self) -> Result<(TokenStream2, TokenStream2, String), String> {
        let object_name = self.ident.clone();
        let mut field_getters = quote!();
        let mut field_setters = quote!();
        // open docs
        let mut docs = "\n\n## API\n\nThe following properties are available for simulating control algorithms".to_string();
        docs = format!(
            "{}\n\n| Property | Getter | Setter |\n|----------|--------|--------|",
            docs
        );

        for field in self.fields.iter() {
            if let Field::State(_) = field {
                let data = field.data();
                // Getters, Setters (and therefore, docs) are only for Operational and Physical fields
                // for now.
                let att_names: Vec<String> =
                    data.attributes.iter().map(|x| x.name.clone()).collect();
                if !att_names.contains(&"physical".to_string())
                    && !att_names.contains(&"operational".to_string())
                {
                    continue;
                }
                // Docs
                let api_fieldname = field.api_name()?;

                let mut row = format!("| `{}` | Yes  ", api_fieldname);
                if att_names.contains(&"physical".to_string()) {
                    row = format!("{} | Research mode |", row);
                } else {
                    row = format!("{} | Yes |", row);
                }
                docs = format!("{}\n{}", docs, row);

                // Extend getters and setters
                let get = field.api_getter(&object_name)?;
                field_getters = quote!(
                    #field_getters
                    #get

                );
                let set = field.api_setter(&object_name)?;
                field_setters = quote!(
                    #field_setters
                    #set
                );
            }
        }

        // return
        Ok((field_getters, field_setters, docs))
    }

    fn get_api(&self, access_from_model: TokenStream2) -> Result<TokenStream2, String> {
        let object_name = self.ident.clone();
        let name_str = format!("{}", &object_name);

        // Register type in API... always within an RC
        let register_type = quote!(
            engine.register_type_with_name::<std::sync::Arc<Self>>(#name_str);
        );

        let (field_getters, field_setters, docs) = self.get_api_getters_setters_docs()?;

        // Return
        let register_api_docs = format!(" Registers the Rhai API for the `{}`", object_name);
        let print_api_docs = format!(" Prints the Rhai API for the `{}`", object_name);
        let r = quote!(
            impl #object_name {

                #[doc = #register_api_docs]
                pub fn register_api(engine : &mut rhai::Engine, model: &std::sync::Arc<Model>, state: &std::sync::Arc<std::cell::RefCell<crate::simulation_state::SimulationState>>, research_mode: bool){

                    #register_type

                    #access_from_model

                    #field_getters

                    #field_setters
                }


                #[cfg(debug_assertions)]
                #[doc = #print_api_docs]
                pub fn print_api_doc(dir: &str, summary: &mut str)->std::io::Result<()>{
                    let api_doc = #docs;
                    let filename = format!("auto-{}.md", #name_str).to_lowercase();
                    let full_filename = format!("{}/{}", dir, filename);

                    let doc = std::fs::read_to_string(full_filename.clone())
                        .expect("Something went wrong reading the documentation file");

                    std::fs::write(&full_filename, format!("{}\n\n{}", doc, api_doc))?;

                    Ok(())
                }

            }
        );
        Ok(r)
    }

    pub fn gen_group_member_api(&self) -> Result<TokenStream2, String> {
        let access_from_model = quote!();
        self.get_api(access_from_model)
    }

    pub fn gen_object_api(&self) -> Result<TokenStream2, String> {
        let object_name = self.ident.clone();
        let name_str = format!("{}", &object_name);
        let name_str_lower = name_str.to_lowercase();

        let location_str = crate::object_location(name_str).unwrap_or_else(|| {
            panic!(
                "Cannot set API for object '{}' which is not stored in the Model",
                &object_name
            )
        });
        let location = syn::Ident::new(location_str, proc_macro2::Span::call_site());

        // register_access_from_model
        let not_found_err = format!("Could not find {} '{{}}'", object_name);
        let out_of_bounds_err = format!(
            "Trying to access {} number {{}}... but the last index is {{}}",
            object_name
        );
        let negative_index_err = format!(
            "Impossible to get {} using a negative index ({{}} was given)",
            object_name
        );
        let access_from_model = quote!(
            // get by name
            let new_mod = std::sync::Arc::clone(model);
            let new_state = std::sync::Arc::clone(state);
            engine.register_fn(#name_str_lower, move |name: &str | -> Result<_, Box<rhai::EvalAltResult>> {
                for s in new_mod.#location.iter(){
                    if s.name == name {
                        return Ok(std::sync::Arc::clone(s))
                    }
                }
                return Err(format!(#not_found_err, name).into());
            });

            // Get by index
            let new_mod = std::sync::Arc::clone(model);
            engine.register_fn(#name_str_lower, move |index: rhai::INT| -> Result<_, Box<rhai::EvalAltResult>> {

                let len = new_mod.#location.len();
                if index < 0 {
                    return Err(format!(#negative_index_err, index).into())
                }
                if index >= len as i64 {
                    return Err(format!(#out_of_bounds_err, index, len - 1).into());
                }
                Ok(std::sync::Arc::clone(&new_mod.#location[index as usize]))
            });

        );

        self.get_api(access_from_model)
    }
}
