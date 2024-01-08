use askama::Template;
use teo_parser::r#type::Type;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::traits::named::Named;
use crate::outline::outline::{Mode, Outline};
use crate::shared::ts::conf::TsConf;
use crate::shared::ts::lookup::lookup;
use crate::utils::filters;

#[derive(Template)]
#[template(path = "shared/ts/index.d.ts.jinja", escape = "none")]
pub(crate) struct TsIndexDTsTemplate<'a> {
    pub(crate) main_namespace: &'a Namespace,
    pub(crate) conf: &'a TsConf,
    pub(crate) mode: Mode,
    pub(crate) render_namespace: &'static dyn Fn(&Namespace, &TsConf, &Namespace, Mode) -> String,
}

unsafe impl Send for TsIndexDTsTemplate<'_> { }
unsafe impl Sync for TsIndexDTsTemplate<'_> { }

#[derive(Template)]
#[template(path = "shared/ts/namespace.partial.jinja", escape = "none")]
pub(crate) struct TsNamespaceTemplate<'a> {
    pub(self) conf: &'a TsConf,
    pub(self) namespace: &'a Namespace,
    pub(self) render_namespace: &'static dyn Fn(&Namespace, &TsConf, &Namespace, Mode) -> String,
    pub(self) outline: &'a Outline,
    pub(self) lookup: &'static dyn Fn(&Type, bool) -> Result<String>,
    pub(self) get_payload_suffix: &'static dyn Fn(&Type) -> &'static str,
    pub(self) ts_extends: &'static dyn Fn(&Vec<Type>) -> String,
    pub(self) main_namespace: &'a Namespace,
    pub(self) mode: Mode,
}

unsafe impl Send for TsNamespaceTemplate<'_> { }
unsafe impl Sync for TsNamespaceTemplate<'_> { }

fn ts_extends(extends: &Vec<Type>) -> String {
    if extends.is_empty() {
        "".to_owned()
    } else {
        extends.iter().map(|extend| lookup(extend, false).unwrap() + " & ").collect::<Vec<String>>().join("")
    }
}

fn get_payload_suffix(t: &Type) -> &'static str {
    if t.is_array() {
        "[]"
    } else if t.is_optional() {
        "?"
    } else {
        ""
    }
}

pub(crate) fn render_namespace(namespace: &Namespace, conf: &TsConf, main_namespace: &Namespace, mode: Mode) -> String {
    let content = TsNamespaceTemplate {
        conf,
        namespace,
        render_namespace: &render_namespace,
        outline: &Outline::new(namespace, mode, main_namespace),
        lookup: &lookup,
        get_payload_suffix: &get_payload_suffix,
        ts_extends: &ts_extends,
        main_namespace,
        mode,
    }.render().unwrap();
    if namespace.path.is_empty() {
        content
    } else {
        format!("export namespace {} {{\n", namespace.name()) + &indent::indent_by(4, content.as_str()) + "\n}"
    }
}