use std::path::Path;
use inflector::Inflector;
use teo_runtime::config::client::Client;

pub(crate) trait ClientExt {

    fn class_name(&self) -> String;

    fn inferred_package_name(&self) -> &str;

    fn inferred_package_name_snake_case(&self) -> String;

    fn inferred_package_name_camel_case(&self) -> String;
}

impl ClientExt for Client {

    fn class_name(&self) -> String {
        let first_char = self.object_name.chars().nth(0).unwrap();
        if first_char.is_uppercase() {
            format!("{}Class", self.object_name)
        } else {
            format!("{}{}", self.object_name.chars().nth(0).unwrap().to_uppercase(), &self.object_name[1..])
        }
    }

    /// # Inferred package name
    ///
    /// Infer the package name from last path component
    fn inferred_package_name(&self) -> &str {
        Path::new(self.dest.as_str()).file_name().map(|s| s.to_str().unwrap()).unwrap_or("untitled")
    }

    fn inferred_package_name_snake_case(&self) -> String {
        self.inferred_package_name().to_snake_case()
    }

    fn inferred_package_name_camel_case(&self) -> String {
        self.inferred_package_name().to_camel_case()
    }
}
