use crate::outline::r#enum::Member;

pub(crate) struct Enum {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) path: Vec<String>,
    pub(crate) name: String,
    pub(crate) members: Vec<Member>,
}

impl Enum {

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

    pub(crate) fn members(&self) -> &Vec<Member> {
        &self.members
    }

    pub(crate) fn joined_enum_variant_names_for_ts(&self) -> String {
        if self.members().is_empty() {
            "undefined".to_owned()
        } else {
            self.members.iter().map(|m| format!("\"{}\"", m.name)).collect::<Vec<String>>().join(" | ")
        }
    }
}