pub struct NamespaceItem {
    pub(crate) name: String,
    pub(crate) path: Vec<String>,
    pub(crate) property_name: String,
}

impl NamespaceItem {

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