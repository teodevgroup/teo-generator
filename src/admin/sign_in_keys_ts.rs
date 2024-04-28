use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::utils::file::FileUtil;

struct AccountModelField {
    name: String
}

struct AccountModel {
    name: String,
    name_lowercase: String,
    id_fields: Vec<AccountModelField>,
    checker_fields: Vec<AccountModelField>,
}

#[derive(Template)]
#[template(path = "admin/lib/generated/signIn/keys.ts.jinja", escape = "none")]
pub(self) struct SignInKeysTsTemplate {
    pub(self) account_models: Vec<AccountModel>,
}

fn fetch_template_data(namespace: &Namespace) -> SignInKeysTsTemplate {
    SignInKeysTsTemplate {
        account_models: namespace.collect_models(|m| m.data.get("admin:administrator").is_some()).iter().map(|m| {
            AccountModel {
                name: m.path().join("."),
                name_lowercase: m.path().iter().map(|s| s.to_camel_case()).join("."),
                id_fields: m.fields().iter().filter(|f| f.data.get("identity:id").is_some()).map(|f| {
                    AccountModelField {
                        name: f.name().to_string()
                    }
                }).collect(),
                checker_fields: m.fields().iter().filter(|f| f.data.get("identity:checker").is_some()).map(|f| {
                    AccountModelField {
                        name: f.name().to_string()
                    }
                }).collect(),
            }
        }).collect()
    }
}

pub(crate) async fn generate_sign_in_keys_ts(namespace: &Namespace, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/lib/generated/signIn/keys.ts", template.render().unwrap()).await?;
    Ok(())
}