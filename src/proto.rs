use extism_pdk::*;
use proto_pdk::*;
use proto_schema_plugin::{PlatformMapper, Schema, SchemaType};
use serde_json::Value as JsonValue;
use starbase_utils::toml;
use std::path::PathBuf;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

fn get_schema() -> Result<Schema, Error> {
    let data = config::get("schema").expect("Missing schema!");
    let schema: Schema = toml::from_str(&data)?;

    Ok(schema)
}

fn get_platform<'schema>(
    schema: &'schema Schema,
    env: &Environment,
) -> Result<&'schema PlatformMapper, PluginError> {
    let os = env.os.to_string();
    let mut platform = schema.platform.get(&os);

    // Fallback to linux for other OSes
    if platform.is_none()
        && (env.os == HostOS::FreeBSD || env.os == HostOS::NetBSD || env.os == HostOS::OpenBSD)
    {
        platform = schema.platform.get("linux");
    }

    platform.ok_or_else(|| PluginError::UnsupportedOS {
        tool: schema.name.clone(),
        os,
    })
}

fn get_bin_path(platform: &PlatformMapper, env: &Environment) -> PathBuf {
    platform
        .bin_path
        .clone()
        .unwrap_or_else(|| format_bin_name(&env.id, env.os))
        .into()
}

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    let schema = get_schema()?;

    Ok(Json(ToolMetadataOutput {
        name: schema.name,
        type_of: match schema.type_of {
            SchemaType::Cli => PluginType::CLI,
            SchemaType::DependencyManager => PluginType::DependencyManager,
            SchemaType::Language => PluginType::Language,
        },
        ..ToolMetadataOutput::default()
    }))
}

fn is_musl() -> bool {
    if cfg!(macos) {
        false
    } else {
        unsafe {
            match exec_command(Json(ExecCommandInput::pipe("ldd", ["--version"]))) {
                Ok(res) => res.0.stdout.contains("musl"),
                Err(_) => false,
            }
        }
    }
}

fn interpolate_tokens(value: &str, schema: &Schema, env: &Environment) -> String {
    let arch = env.arch.to_rust_arch();
    let os = env.os.to_string();

    let mut value = value
        .replace("{version}", &env.version)
        .replace("{arch}", schema.install.arch.get(&arch).unwrap_or(&arch))
        .replace("{os}", &os);

    // Avoid detecting musl unless requested
    if value.contains("{libc}") {
        value = value.replace("{libc}", if is_musl() { "musl" } else { "gnu" });
    }

    value
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let schema = get_schema()?;
    let platform = get_platform(&schema, &input.env)?;

    let download_file = interpolate_tokens(&platform.download_file, &schema, &input.env);
    let download_url = interpolate_tokens(&schema.install.download_url, &schema, &input.env)
        .replace("{download_file}", &download_file);

    let checksum_file = interpolate_tokens(
        platform
            .checksum_file
            .as_ref()
            .unwrap_or(&"CHECKSUM.txt".to_string()),
        &schema,
        &input.env,
    );
    let checksum_url = schema.install.checksum_url.as_ref().map(|url| {
        interpolate_tokens(&url, &schema, &input.env).replace("{checksum_file}", &checksum_file)
    });

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: platform.archive_prefix.clone(),
        checksum_url,
        checksum_name: Some(checksum_file),
        download_url,
        download_name: Some(download_file),
    }))
}

#[plugin_fn]
pub fn locate_bins(Json(input): Json<LocateBinsInput>) -> FnResult<Json<LocateBinsOutput>> {
    let schema = get_schema()?;
    let platform = get_platform(&schema, &input.env)?;

    Ok(Json(LocateBinsOutput {
        bin_path: Some(get_bin_path(platform, &input.env)),
        fallback_last_globals_dir: true,
        globals_lookup_dirs: schema.globals.lookup_dirs,
        globals_prefix: schema.globals.package_prefix,
        ..LocateBinsOutput::default()
    }))
}

pub fn remove_v_prefix(value: &str) -> &str {
    if value.starts_with('v') || value.starts_with('V') {
        return &value[1..];
    }

    value
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let schema = get_schema()?;

    if let Some(repository) = schema.resolve.git_url {
        let pattern = regex::Regex::new(&schema.resolve.git_tag_pattern)?;

        let tags = load_git_tags(&repository)?
            .into_iter()
            .filter_map(|t| {
                pattern
                    .captures(&t)
                    .map(|captures| remove_v_prefix(captures.get(1).unwrap().as_str()).to_string())
            })
            .collect::<Vec<_>>();

        return Ok(Json(LoadVersionsOutput::from_tags(&tags)?));
    }

    if let Some(endpoint) = schema.resolve.manifest_url {
        let response: Vec<JsonValue> = fetch_url_with_cache(endpoint)?;
        let version_key = &schema.resolve.manifest_version_key;
        let mut versions = vec![];

        for row in response {
            match row {
                JsonValue::String(v) => {
                    versions.push(remove_v_prefix(&v).to_string());
                }
                JsonValue::Object(o) => {
                    if let Some(JsonValue::String(v)) = o.get(version_key) {
                        versions.push(remove_v_prefix(&v).to_string());
                    }
                }
                _ => {}
            }
        }

        return Ok(Json(LoadVersionsOutput::from_tags(&versions)?));
    }

    err!(
        "Unable to resolve versions for {}. Schema either requires a `git_url` or `manifest_url`."
            .into()
    )
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    let mut output = DetectVersionOutput::default();
    let schema = get_schema()?;

    if let Some(files) = schema.detect.version_files {
        output.files = files;
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn create_shims(Json(input): Json<CreateShimsInput>) -> FnResult<Json<CreateShimsOutput>> {
    let mut output = CreateShimsOutput::default();
    let schema = get_schema()?;
    let platform = get_platform(&schema, &input.env)?;
    let bin_path = get_bin_path(platform, &input.env);

    output.no_primary_global = !schema.shim.global;

    if schema.shim.local {
        output.local_shims.insert(
            input.env.id,
            if let Some(parent_bin) = schema.shim.parent_bin {
                ShimConfig::local_with_parent(bin_path, parent_bin)
            } else {
                ShimConfig::local(bin_path)
            },
        );
    }

    Ok(Json(output))
}
