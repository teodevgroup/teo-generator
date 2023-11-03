use teo_parser::r#type::Type;

pub(in crate::entity) struct Field {
    pub(in crate::entity) title: String,
    pub(in crate::entity) desc: String,
    pub(in crate::entity) name: String,
    pub(in crate::entity) r#type: Type,
}