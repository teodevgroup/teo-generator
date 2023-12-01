pub struct GroupItem {
    pub(crate) name: String,
    pub(crate) path: Vec<String>,
}

impl GroupItem {

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }
}