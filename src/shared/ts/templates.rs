use askama::Template;
use teo_parser::r#type::Type;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::traits::named::Named;
use crate::outline::outline::{Mode, Outline};
use crate::shared::ts::conf::TsConf;
use crate::shared::ts::lookup::lookup;
use crate::utils::filters;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::model::field::typed::Typed;

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
    pub(self) lookup: &'static dyn Fn(&Type, bool, Mode) -> Result<String>,
    pub(self) get_payload_suffix: &'static dyn Fn(&Type) -> &'static str,
    pub(self) ts_extends: &'static dyn Fn(&Vec<Type>, Mode) -> String,
    pub(self) main_namespace: &'a Namespace,
    pub(self) mode: Mode,
    pub(self) optional_strategy: &'static dyn Fn(&String) -> String,
    pub(self) group_by_generics: &'static dyn Fn(String) -> String,
}

unsafe impl Send for TsNamespaceTemplate<'_> { }
unsafe impl Sync for TsNamespaceTemplate<'_> { }

fn ts_extends(extends: &Vec<Type>, mode: Mode) -> String {
    if extends.is_empty() {
        "".to_owned()
    } else {
        extends.iter().map(|extend| lookup(extend, false, mode).unwrap() + " & ").collect::<Vec<String>>().join("")
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

fn group_by_generics(original: String) -> String {
    format!(r#"T extends {original},
      HasSelectOrTake extends Or<
        Extends<'skip', Keys<T>>,
        Extends<'take', Keys<T>>
      >,
      OrderByArg extends True extends HasSelectOrTake
        ? {{ orderBy: {original}['orderBy'] }}
        : {{ orderBy?: {original}['orderBy'] }},
      OrderFields extends ExcludeUnderscoreKeys<Keys<MaybeTupleToUnion<T['orderBy']>>>,
      ByFields extends MaybeTupleToUnion<T['by']>,
      ByValid extends Has<ByFields, OrderFields>,
      HavingFields extends GetHavingFields<T['having']>,
      HavingValid extends Has<ByFields, HavingFields>,
      ByEmpty extends T['by'] extends never[] ? True : False,
      InputErrors extends ByEmpty extends True
      ? `Error: "by" must not be empty.`
      : HavingValid extends False
      ? {{
          [P in HavingFields]: P extends ByFields
            ? never
            : P extends string
            ? `Error: Field "${{P}}" used in "having" needs to be provided in "by".`
            : [
                Error,
                'Field ',
                P,
                ` in "having" needs to be provided in "by"`,
              ]
        }}[HavingFields]
      : 'take' extends Keys<T>
      ? 'orderBy' extends Keys<T>
        ? ByValid extends True
          ? {{}}
          : {{
              [P in OrderFields]: P extends ByFields
                ? never
                : `Error: Field "${{P}}" in "orderBy" needs to be provided in "by"`
            }}[OrderFields]
        : 'Error: If you provide "take", you also need to provide "orderBy"'
      : 'skip' extends Keys<T>
      ? 'orderBy' extends Keys<T>
        ? ByValid extends True
          ? {{}}
          : {{
              [P in OrderFields]: P extends ByFields
                ? never
                : `Error: Field "${{P}}" in "orderBy" needs to be provided in "by"`
            }}[OrderFields]
        : 'Error: If you provide "skip", you also need to provide "orderBy"'
      : ByValid extends True
      ? {{}}
      : {{
          [P in OrderFields]: P extends ByFields
            ? never
            : `Error: Field "${{P}}" in "orderBy" needs to be provided in "by"`
        }}[OrderFields]"#)
}

fn optional_strategy(original: &String) -> String {
    if original.ends_with("?") {
        original[0..original.len() - 1].to_owned() + " | null"
    } else {
        original.clone()
    }
}

pub(crate) fn render_namespace(namespace: &Namespace, conf: &TsConf, main_namespace: &Namespace, mode: Mode) -> String {
    let content = TsNamespaceTemplate {
        conf,
        namespace,
        render_namespace: &render_namespace,
        outline: &Outline::new(namespace, mode, main_namespace, false),
        lookup: &lookup,
        get_payload_suffix: &get_payload_suffix,
        ts_extends: &ts_extends,
        main_namespace,
        mode,
        optional_strategy: &optional_strategy,
        group_by_generics: &group_by_generics,
    }.render().unwrap();
    if namespace.path.is_empty() {
        content
    } else {
        format!("export namespace {} {{\n", namespace.name()) + &indent::indent_by(4, content.as_str()) + "\n}"
    }
}