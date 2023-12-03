use teo_parser::r#type::Type;

pub struct RequestItem {
    pub name: String,
    pub input_type: Type,
    pub output_type: Type,
    pub has_custom_url_args: bool,
    pub is_form: bool,
    pub has_body_input: bool,
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

    pub fn has_custom_url_args(&self) -> bool {
        self.has_custom_url_args
    }

    pub fn is_form(&self) -> bool {
        self.is_form
    }

    pub fn has_body_input(&self) -> bool {
        self.has_body_input
    }
}