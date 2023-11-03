use crate::entity::outline::r#enum::Member;

pub(in crate::entity) struct Enum {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) path: Vec<String>,
    pub(in super::super) name: String,
    pub(in super::super) members: Vec<Member>,
}

impl Enum {

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

    pub(in crate::entity) fn members(&self) -> &Vec<Member> {
        &self.members
    }
}