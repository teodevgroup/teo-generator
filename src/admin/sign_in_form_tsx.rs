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
#[template(path = "admin/components/generated/signInModal/SignInForm.tsx.jinja", escape = "none")]
pub(self) struct SignInFormTsxTemplate {
    pub(self) imports: String, // useSignInAdminDefaultCheckerKey, useSignInAdminDefaultIdKey, useSignInUserDefaultCheckerKey, useSignInUserDefaultIdKey
}

fn fetch_template_data(namespace: &Namespace) -> SignInFormTsxTemplate {
    let models = namespace.collect_models(|m| m.data.get("admin:administrator").is_some());
    let imports = models.iter().map(|m| {
        let concat = m.path().join("");
        format!("useSignIn{}DefaultCheckerKey, useSignIn{}DefaultIdKey", concat, concat)
    }).join(", ");

    SignInFormTsxTemplate {
        imports,

    }
}

pub(crate) async fn generate_sign_in_form_tsx(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/components/generated/signInModal/SignInForm.tsx", template.render().unwrap()).await?;
    Ok(())
}