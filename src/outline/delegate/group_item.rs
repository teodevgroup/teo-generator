pub struct GroupItem {
    pub(crate) name: String,
    pub(crate) path: Vec<String>,
    pub(crate) property_name: String,
}

impl GroupItem {

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn property_name(&self) -> &str {
        &self.property_name
    }
}