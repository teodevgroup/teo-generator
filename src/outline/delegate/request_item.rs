use teo_parser::r#type::Type;

pub struct RequestItem {
    pub name: String,
    pub input_type: Type,
    pub output_type: Type,
}

impl RequestItem {

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn input_type(&self) -> &Type {
        &self.input_type
    }

    pub fn output_type(&self) -> &Type {
        &self.output_type
    }
}