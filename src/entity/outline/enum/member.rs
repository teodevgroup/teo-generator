pub(in crate::entity) struct Member {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) name: String,
}

impl Member {

    pub(in crate::entity) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(in crate::entity) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(in crate::entity) fn name(&self) -> &str {
        self.name.as_str()
    }
}