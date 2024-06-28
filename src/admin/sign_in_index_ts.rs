use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use crate::utils::file::FileUtil;

struct AccountItem {
    name: String,
    path: String,
}

#[derive(Template)]
#[template(path = "admin/src/lib/generated/signIn/index.ts.jinja", escape = "none")]
pub(self) struct SignInIndexTsTemplate {
    pub(self) imports: String, // Admin, User
    pub(self) account_models_type: String, // "Admin" | "User"
    pub(self) account_models_list: String, // "Admin", "User"
    pub(self) account_data: String, // {
    //     "Admin": std.DataMeta<Admin, std.identity.TokenInfo>
    // } | {
    //     "User": std.DataMeta<User, std.identity.TokenInfo>
    // }
    pub(self) account_items: Vec<AccountItem>,
}

fn fetch_template_data(namespace: &Namespace) -> SignInIndexTsTemplate {
    let models = namespace.collect_models(|m| m.data().get("admin:administrator").is_some());
    let imports = models.iter().map(|m| m.path().first().unwrap().to_string()).join(", ");
    let account_models_type = models.iter().map(|m| format!("\"{}\"", m.path().join("."))).join(" | ");
    let account_models_list = models.iter().map(|m| format!("\"{}\"", m.path().join("."))).join(", ");
    let account_data = models.iter().map(|m| {
        let pathed = m.path().join(".");
        let wrapped = format!("\"{}\"", pathed);
        format!("{{\n    {}: std.DataMeta<{}, std.identity.TokenInfo>\n}}", wrapped, pathed)
    }).join(" | ");
    let account_items = models.iter().map(|m| {
        AccountItem {
            name: m.path().join("."),
            path: m.path().iter().map(|s| s.to_camel_case()).join("."),
        }
    }).collect();
    SignInIndexTsTemplate {
        imports,
        account_models_type,
        account_models_list,
        account_data,
        account_items
    }
}

pub(crate) async fn generate_sign_in_index_ts(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/lib/generated/signIn/index.ts", template.render().unwrap()).await?;
    Ok(())
}