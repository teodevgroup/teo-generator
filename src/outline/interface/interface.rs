use teo_parser::r#type::Type;
use crate::outline::interface::Field;

pub(crate) struct Interface {
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

    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(crate) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(crate) fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn generic_names(&self) -> &Vec<String> {
        &self.generic_names
    }

    pub(crate) fn extends(&self) -> &Vec<Type> {
        &self.extends
    }

    pub(crate) fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    pub(crate) fn synthesized(&self) -> &Option<(String, Option<String>)> {
        &self.synthesized
    }
}