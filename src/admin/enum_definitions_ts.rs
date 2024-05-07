use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use teo_result::Result;
use teo_runtime::traits::named::Named;

struct EnumDefinition {
    name_pascalcase: String,
    name_camelcase: String,
    members: Vec<String>,
}

#[derive(Template)]
#[template(path = "admin/src/lib/generated/enumDefinitions.ts.jinja", escape = "none")]
pub(self) struct EnumDefinitionsTsTemplate {
    pub(self) enums: Vec<EnumDefinition>,
}

fn fetch_template_data(namespace: &Namespace) -> EnumDefinitionsTsTemplate {
    let enums = namespace.collect_enums(|e| !e.interface);
    EnumDefinitionsTsTemplate {
        enums: enums.iter().map(|e| EnumDefinition {
            name_pascalcase: e.path().iter().join("."),
            name_camelcase: e.path().iter().map(|s| s.to_camel_case()).join("."),
            members: e.members().iter().map(|m| m.name().to_owned()).collect(),
        }).collect()
    }
}

pub(crate) async fn generate_enum_definitions_ts(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/lib/generated/enumDefinitions.ts", template.render().unwrap()).await?;
    Ok(())
}