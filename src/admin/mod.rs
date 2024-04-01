use teo_runtime::config::admin::Admin;
use teo_runtime::namespace::Namespace;
use serde::Deserialize;
use crate::utils::file::FileUtil;

static FILE_ADDRESS: &'static str = "https://raw.githubusercontent.com/teocloud/teo-admin-dev/main/";
static FILE_JSON: &'static str = ".generator/data/fileList.json";

#[derive(Deserialize)]
struct FileList {
    generated: Vec<String>,
    extended: Vec<String>,
}

pub async fn generate(main_namespace: &Namespace, admin: &Admin) -> teo_result::Result<()> {
    let dest_dir = std::env::current_dir()?.join(admin.dest.as_str());
    let file_util = FileUtil::new(dest_dir);
    file_util.ensure_root_directory().await?;
    let file_list = reqwest::get(FILE_ADDRESS.to_owned() + FILE_JSON)
        .await?
        .json::<FileList>()
        .await?;
    for generated in &file_list.generated {
        println!("generated {generated}");
    }
    Ok(())
}