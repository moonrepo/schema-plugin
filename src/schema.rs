use proto_pdk::{ExecutableConfig, HostArch, HostLibc, HostOS};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PlatformMapper {
    pub archs: Vec<HostArch>,
    pub archive_prefix: Option<String>,
    pub bin_path: Option<String>,
    pub checksum_file: Option<String>,
    pub download_file: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DetectSchema {
    pub version_files: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct InstallSchema {
    pub arch: HashMap<HostArch, String>,
    pub libc: HashMap<HostLibc, String>,
    pub checksum_public_key: Option<String>,
    pub checksum_url: Option<String>,
    pub checksum_url_canary: Option<String>,
    pub download_url: String,
    pub download_url_canary: Option<String>,

    // Primary
    pub primary: Option<ExecutableConfig>,
    pub no_bin: Option<bool>,
    pub no_shim: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PackagesSchema {
    pub globals_lookup_dirs: Vec<String>,
    pub globals_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ResolveSchema {
    pub version_pattern: String,
    // Manifest
    pub manifest_url: Option<String>,
    pub manifest_version_key: String,
    // Tags
    pub git_url: Option<String>,
    pub git_tag_pattern: Option<String>,
}

impl Default for ResolveSchema {
    fn default() -> Self {
        ResolveSchema {
            manifest_url: None,
            manifest_version_key: "version".to_string(),
            git_url: None,
            git_tag_pattern: None,
            version_pattern:
                r"^v?((?<major>[0-9]+)\.(?<minor>[0-9]+)\.(?<patch>[0-9]+)(?<pre>-[0-9a-zA-Z\.]+)?(?<build>\+[-0-9a-zA-Z\.]+)?)$"
                    .to_string(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct MetadataSchema {
    pub self_upgrade_commands: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaType {
    #[default]
    Language,
    #[serde(alias = "package-manager")]
    DependencyManager,
    Cli,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Schema {
    pub name: String,
    #[serde(rename = "type")]
    pub type_of: SchemaType,
    pub metadata: MetadataSchema,
    pub platform: HashMap<HostOS, PlatformMapper>,

    pub detect: DetectSchema,
    pub install: InstallSchema,
    pub packages: PackagesSchema,
    pub resolve: ResolveSchema,
}
