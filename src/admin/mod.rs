pub mod sign_in_index_ts;
pub mod sign_in_keys_ts;
pub mod preferences_ts;
pub mod default_preferences_ts;
pub mod sign_in_form_tsx;
pub mod translations_index_ts;
pub mod translations_init_ts;
pub mod translations_lang_index_ts;
pub mod translations_languages_ts;
pub mod pages_index_index_ts;
pub mod pages_page_stack_default_item_keys_tsx;
pub mod pages_render_default_stack_item_tsx;
pub mod pages_page_dashboard;
pub mod pages_page_form;
pub mod pages_page_form_page;
pub mod pages_page_index;
pub mod pages_page_records;
pub mod pages_page_records_list;
pub mod webpack_config_ts;
pub mod enum_definitions_ts;

use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::config::admin::Admin;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use serde::Deserialize;
use serde_json::json;
use teo_runtime::config::client::{Client, ClientLanguage, TypeScriptHTTPProvider};
use teo_runtime::config::server::Server;
use once_cell::sync::Lazy;
use crate::admin::default_preferences_ts::generate_default_preferences_ts;
use crate::admin::enum_definitions_ts::generate_enum_definitions_ts;
use crate::admin::pages_index_index_ts::generate_pages_index_index_ts;
use crate::admin::pages_page_dashboard::generate_pages_page_dashboard_tsx;
use crate::admin::pages_page_form::generate_pages_page_form_tsx;
use crate::admin::pages_page_form_page::generate_pages_page_form_page_tsx;
use crate::admin::pages_page_index::generate_pages_page_index_tsx;
use crate::admin::pages_page_records::generate_pages_page_records_tsx;
use crate::admin::pages_page_records_list::generate_pages_page_records_list_tsx;
use crate::admin::pages_page_stack_default_item_keys_tsx::generate_pages_stack_default_item_keys_tsx;
use crate::admin::pages_render_default_stack_item_tsx::generate_pages_render_default_stack_item_tsx;
use crate::admin::preferences_ts::generate_preferences_ts;
use crate::admin::sign_in_form_tsx::generate_sign_in_form_tsx;
use crate::admin::sign_in_index_ts::generate_sign_in_index_ts;
use crate::admin::sign_in_keys_ts::generate_sign_in_keys_ts;
use crate::admin::translations_index_ts::generate_translations_index_ts;
use crate::admin::translations_init_ts::generate_translations_init_ts;
use crate::admin::translations_lang_index_ts::generate_translations_lang_index_ts;
use crate::admin::translations_languages_ts::generate_translations_languages_ts;
use crate::admin::webpack_config_ts::generate_webpack_config_ts;
use crate::utils::file::FileUtil;
use crate::utils::update_package_json_version::update_package_json_version;

static FILE_ADDRESS: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(Box::new(format!("https://raw.githubusercontent.com/teodevgroup/teo-admin-dev/{}/", env!("CARGO_PKG_VERSION"))))
});

static FILE_JSON: &'static str = ".generator/data/fileList.json";

#[derive(Deserialize)]
struct FileList {
    generated: Vec<String>,
    extended: Vec<String>,
}

pub async fn generate(main_namespace: &Namespace, admin: &Admin, server: &Server) -> Result<()> {
    let dest_dir = std::env::current_dir()?.join(admin.dest.as_str());
    let file_util = FileUtil::new(dest_dir.clone());
    file_util.ensure_root_directory().await?;
    // download remote sources
    let file_list = reqwest::get(FILE_ADDRESS.to_owned() + FILE_JSON)
        .await?
        .json::<FileList>()
        .await?;
    for extended_file in &file_list.extended {
        let file_location = dest_dir.join(extended_file);
        if !file_location.exists() {
            create_file_from_remote_source(extended_file, &file_util).await?;
        }
    }
    for generated_file in &file_list.generated {
        create_file_from_remote_source(generated_file, &file_util).await?;
    }
    // ensure custom directories
    let custom_lib = dest_dir.as_path().join("src/lib/custom");
    let custom_components = dest_dir.as_path().join("src/components/custom");
    file_util.ensure_directory(custom_lib.as_os_str().to_str().unwrap()).await?;
    file_util.ensure_directory(custom_components.as_os_str().to_str().unwrap()).await?;

    // dynamic generated files

    // sign in
    generate_sign_in_index_ts(main_namespace, &file_util).await?;
    generate_sign_in_keys_ts(main_namespace, &file_util).await?;
    generate_sign_in_form_tsx(main_namespace, &file_util).await?;

    // preferences
    generate_preferences_ts(main_namespace, &file_util).await?;
    generate_default_preferences_ts(main_namespace, &admin.languages, &file_util).await?;

    // enum definitions
    generate_enum_definitions_ts(main_namespace, &file_util).await?;

    // language
    // generated
    create_file_from_remote_source("src/lib/generated/translations/static.ts", &file_util).await?;
    generate_translations_index_ts(main_namespace, &file_util).await?;
    generate_translations_init_ts(&admin.languages, &file_util).await?;
    generate_translations_languages_ts(&admin.languages, &file_util).await?;
    // extended
    let index_ts = "src/lib/extended/translations/index.ts";
    let file_location = dest_dir.join(index_ts);
    if !file_location.exists() {
        create_file_from_remote_source(index_ts, &file_util).await?;
    }
    for lang in admin.languages.iter() {
        // generated
        create_file_from_remote_source(&format!("src/lib/generated/translations/{}/static.ts", lang.as_str()), &file_util).await?;
        generate_translations_lang_index_ts(lang.as_str(), main_namespace, &file_util).await?;
        // extended
        let location = dest_dir.join(format!("src/lib/extended/translations/{}.ts", lang.as_str()));
        if !file_location.exists() {
            create_file_from_remote_source(location.to_str().unwrap(), &file_util).await?;
        }
    }

    // -- pages

    // _Index
    generate_pages_index_index_ts(main_namespace, &file_util).await?;

    // Common
    generate_pages_stack_default_item_keys_tsx(main_namespace, &file_util).await?;
    generate_pages_render_default_stack_item_tsx(main_namespace, &file_util).await?;

    // Model
    for m in main_namespace.collect_models(|m| m.data().get("admin:ignore").is_none()) {
        let model_variable_name = m.path().iter().map(|s| s.to_pascal_case()).join("");
        let path = m.path().join("/");
        generate_pages_page_dashboard_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
        generate_pages_page_form_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
        generate_pages_page_form_page_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
        generate_pages_page_index_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
        generate_pages_page_records_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
        generate_pages_page_records_list_tsx(main_namespace, m, &model_variable_name, &path, &file_util).await?;
    }
    // readme
    file_util.generate_file("README.md", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/admin/readme.md.jinja"))).await?;

    // webpack.config.ts
    generate_webpack_config_ts(server.bind.1.to_string(), &file_util).await?;

    // package.json
    let remote_json_string = fetch_remote_content("package.json").await?;
    let remote_json_data: serde_json::Value = serde_json::from_str(&remote_json_string).unwrap();
    let dependencies = remote_json_data.get("dependencies").unwrap();
    let mut dev_dependencies = remote_json_data.get("devDependencies").unwrap().clone();
    dev_dependencies.as_object_mut().unwrap().shift_remove("glob").unwrap();
    let new_json_data = json!({
        "name": "admin-dashboard",
        "version": "0.0.1",
        "description": "This project is generated with Teo.",
        "private": true,
        "scripts": {
            "start": "npx webpack-dev-server",
        },
        "dependencies": dependencies.clone(),
        "devDependencies": dev_dependencies.clone(),
    });
    if file_util.generate_file_if_not_exist("package.json", serde_json::to_string(&new_json_data).unwrap()).await? {
        // if exists, update package.json with a minor version and deps
        let json_data = std::fs::read_to_string(file_util.get_file_path("package.json"))
            .expect("Unable to read package.json");
        file_util.generate_file("package.json", update_json_version_and_deps(json_data, dependencies, &dev_dependencies)).await?;
    }
    // generate TypeScript client
    crate::client::generate(main_namespace, &Client {
        provider: ClientLanguage::TypeScript(TypeScriptHTTPProvider::Fetch),
        dest: dest_dir.as_path().join("src/lib/generated/teo").to_str().unwrap().to_owned(),
        package: false,
        host: admin.host.clone(),
        object_name: "teo".to_owned(),
        git_commit: false,
    }).await?;
    Ok(())
}

fn update_json_version_and_deps(json_data: String, dependencies: &serde_json::Value, dev_dependencies: &serde_json::Value) -> String {
    let version_updated_json_data = update_package_json_version(json_data);
    let mut json_value: serde_json::Value = serde_json::from_str(&version_updated_json_data).unwrap();
    let deps = json_value.get_mut("dependencies").unwrap();
    let deps_object = deps.as_object_mut().unwrap();
    let source_deps = dependencies.as_object().unwrap();
    for (k, v) in source_deps {
        if deps_object.get(k).is_none() {
            deps_object.insert(k.to_owned(), v.clone());
        }
    }
    let dev_deps = json_value.get_mut("devDependencies").unwrap();
    let dev_deps_object = dev_deps.as_object_mut().unwrap();
    let source_dev_deps = dev_dependencies.as_object().unwrap();
    for (k, v) in source_dev_deps {
        if dev_deps_object.get(k).is_none() {
            dev_deps_object.insert(k.to_owned(), v.clone());
        }
    }
    serde_json::to_string(&json_value).unwrap()
}

async fn fetch_remote_content(location: &str) -> Result<String> {
    Ok(reqwest::get(FILE_ADDRESS.to_owned() + location)
        .await?
        .text()
        .await?)
}

async fn create_file_from_remote_source(location: &str, file_util: &FileUtil) -> Result<()> {
    let content = fetch_remote_content(location).await?;
    file_util.ensure_directory_and_generate_file(location, content).await
}