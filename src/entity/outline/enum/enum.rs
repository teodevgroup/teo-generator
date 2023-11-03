use crate::entity::outline::r#enum::Member;

pub(in crate::entity) struct Enum {
    pub(in crate::entity) title: String,
    pub(in crate::entity) desc: String,
    pub(in crate::entity) path: Vec<String>,
    pub(in crate::entity) name: String,
    pub(in crate::entity) members: Vec<Member>,
}