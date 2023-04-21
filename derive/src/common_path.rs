pub fn path_to_string(path: &syn::Path) -> String {
    format!("{}", path.segments.iter().next().unwrap().ident)
}

fn path_is(path: &syn::Path, kind: &str) -> bool {
    path.segments.len() == 1 && path_to_string(path) == kind
}

pub fn path_is_option(path: &syn::Path) -> bool {
    path_is(path, "Option")
}

pub fn path_is_vec(path: &syn::Path) -> bool {
    path_is(path, "Vec")
}

pub fn path_is_rc(path: &syn::Path) -> bool {
    path_is(path, "Rc")
}

pub fn path_is_float(path: &syn::Path) -> bool {
    path.segments.len() == 1
        && (path_to_string(path) == "Float"
            || path_to_string(path) == "f32"
            || path_to_string(path) == "f64")
}
pub fn path_is_int(path: &syn::Path) -> bool {
    path_is(path, "usize")
}

pub fn path_is_bool(path: &syn::Path) -> bool {
    path_is(path, "bool")
}

pub fn path_is_string(path: &syn::Path) -> bool {
    path.segments.len() == 1 && path.segments.iter().next().unwrap().ident == "String"
}

// https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn
pub fn extract_type_from_path(path: &syn::Path) -> Result<syn::Type, String> {
    // Get the first segment of the path (there is only one, in fact: "Option"):
    let type_params = &path.segments[0].arguments; //.iter().first().unwrap().arguments;
                                                   // It should have only one angle-bracketed param ("<String>"):
    let generic_arg = match type_params {
        syn::PathArguments::AngleBracketed(params) => &params.args[0], //.iter().first().unwrap(),
        _ => {
            return Err(format!(
                "TODO: error handling 1... found {}",
                path_to_string(path)
            ))
        }
    };
    // This argument must be a type:
    match generic_arg {
        syn::GenericArgument::Type(ty) => Ok(ty.clone()),
        _ => Err(format!(
            "TODO: error handling 2... found {}",
            path_to_string(path)
        )),
    }
}
