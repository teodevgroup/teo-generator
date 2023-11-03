use teo_parser::r#type::Type;
use crate::entity::outline::interface::Field;

pub(in crate::entity) struct Interface {
    pub(in crate::entity) title: String,
    pub(in crate::entity) desc: String,
    pub(in crate::entity) path: Vec<String>,
    pub(in crate::entity) name: String,
    pub(in crate::entity) generic_names: Vec<String>,
    pub(in crate::entity) extends: Vec<Type>,
    pub(in crate::entity) fields: Vec<Field>,
    pub(in crate::entity) synthesized: Option<(String, Option<String>)>,
}