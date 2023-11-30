pub(crate) struct Member {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) name: String,
}

impl Member {

    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(crate) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }
}