use teo_parser::r#type::Type;

pub(in crate::entity) struct Field {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) name: String,
    pub(in super::super) r#type: Type,
}

impl Field {

    pub(in crate::entity) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(in crate::entity) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(in crate::entity) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(in crate::entity) fn r#type(&self) -> &Type {
        &self.r#type
    }
}