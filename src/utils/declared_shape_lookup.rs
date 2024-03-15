use teo_parser::r#type::reference::Reference;
use teo_parser::r#type::Type;
use teo_result::{Result, Error};

pub(crate) fn declared_shape_lookup(
    declared: &Reference,
    owner: &Type,
    path_separator: &str,
) -> Result<String> {
    if let Some(owner) = owner.as_model_object() {
        Ok(owner.string_path().join(path_separator) + declared.string_path().last().unwrap())
    } else {
        Err(Error::new("owner is not model"))
    }
}