use teo_parser::r#type::Type;
use crate::entity::outline::interface::Field;

pub(in crate::entity) struct Interface {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) path: Vec<String>,
    pub(in super::super) name: String,
    pub(in super::super) generic_names: Vec<String>,
    pub(in super::super) extends: Vec<Type>,
    pub(in super::super) fields: Vec<Field>,
    pub(in super::super) synthesized: Option<(String, Option<String>)>,
}

impl Interface {

    pub(in crate::entity) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(in crate::entity) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(in crate::entity) fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub(in crate::entity) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(in crate::entity) fn generic_names(&self) -> &Vec<String> {
        &self.generic_names
    }

    pub(in crate::entity) fn extends(&self) -> &Vec<Type> {
        &self.extends
    }

    pub(in crate::entity) fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    pub(in crate::entity) fn synthesized(&self) -> &Option<(String, Option<String>)> {
        &self.synthesized
    }
}