pub(crate) struct TsConf {
    pub(crate) object_name: String,
    pub(crate) class_name: String,
    is_client: bool,
}

impl TsConf {

    pub fn new(object_name: String, class_name: String, is_client: bool) -> Self {
        TsConf {
            object_name,
            class_name,
            is_client,
        }
    }

    pub fn object_name(&self) -> &str {
        self.object_name.as_str()
    }

    pub fn class_name(&self) -> &str {
        self.class_name.as_str()
    }

    pub fn is_client(&self) -> bool {
        self.is_client
    }

    pub fn is_entity(&self) -> bool {
        !self.is_client
    }
}
