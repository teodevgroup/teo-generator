use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::traits::named::Named;
use crate::admin::preferences_ts::AccountModel;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/src/components/generated/signInModal/SignInForm.tsx.jinja", escape = "none")]
pub(self) struct SignInFormTsxTemplate {
    pub(self) imports: String, // useSignInAdminDefaultCheckerKey, useSignInAdminDefaultIdKey, useSignInUserDefaultCheckerKey, useSignInUserDefaultIdKey
    pub(self) account_models: Vec<AccountModel>,
}

fn fetch_template_data(namespace: &Namespace) -> SignInFormTsxTemplate {
    let models = namespace.collect_models(|m| m.data().get("admin:administrator").is_some());
    let imports = models.iter().map(|m| {
        let concat = m.path().join("");
        format!("useSignIn{}DefaultCheckerKey, useSignIn{}DefaultIdKey", concat, concat)
    }).join(", ");

    SignInFormTsxTemplate {
        imports,
        account_models: models.iter().map(|m| AccountModel {
            pascalcase_name: m.path().iter().join(""),
            camelcase_name: m.path().iter().join("").to_camel_case(),
            secure_fields: m.fields().values().filter(|f| f.data().get("admin:secureInput").is_some()).map(|f| format!("\"{}\"", f.name())).join(", ")
        }).collect()
    }
}

pub(crate) async fn generate_sign_in_form_tsx(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/components/generated/signInModal/SignInForm.tsx", template.render().unwrap()).await?;
    Ok(())
}