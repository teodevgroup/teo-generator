use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

pub(crate) async fn generate_pages_index_index_ts(_namespace: &Namespace, file_util: &FileUtil) -> teo_result::Result<()> {
    file_util.ensure_directory_and_generate_file("src/components/generated/pages/_Index/index.tsx", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/admin/src/components/generated/pages/_Index/index.ts.jinja"))).await?;
    Ok(())
}