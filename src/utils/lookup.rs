use teo_parser::r#type::Type;
use teo_result::Result;

pub(crate) trait Lookup {
    fn call(&self, t: &Type) -> Result<String>;
}

impl<F> Lookup for F where F: Fn(&Type) -> Result<String> {

    fn call(&self, t: &Type) -> Result<String> {
        self(t)
    }
}