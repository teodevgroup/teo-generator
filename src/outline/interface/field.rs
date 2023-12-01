use teo_parser::r#type::synthesized_shape_reference::SynthesizedShapeReferenceKind;
use teo_parser::r#type::Type;

pub(crate) struct Field {
    pub(in super::super) title: String,
    pub(in super::super) desc: String,
    pub(in super::super) name: String,
    pub(in super::super) r#type: Type,
}

impl Field {

    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(crate) fn desc(&self) -> &str {
        self.desc.as_str()
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub(crate) fn is_relation(&self) -> bool {
        self.r#type().unwrap_optional().unwrap_array().unwrap_optional().is_model_object() ||
            (self.r#type().unwrap_optional().unwrap_array().unwrap_optional().is_synthesized_shape_reference() &&
                self.r#type().unwrap_optional().unwrap_array().unwrap_optional().as_synthesized_shape_reference().unwrap().kind == SynthesizedShapeReferenceKind::Result
            )
    }
}