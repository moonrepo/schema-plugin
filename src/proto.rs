use crate::schema::{PlatformMapper, Schema, SchemaType};
use extism_pdk::*;
use proto_pdk::*;
use regex::Captures;
use serde_json::Value as JsonValue;

#[host_fn]
extern "ExtismHost" {
    fn host_log(input: Json<HostLogInput>);
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

fn get_schema() -> Result<Schema, Error> {
    let data = config::get("schema")?.expect("Missing schema!");
    let schema: Schema = json::from_str(&data)?;

    Ok(schema)
}

fn get_platform<'schema>(
    schema: &'schema Schema,
    env: &HostEnvironment,
) -> Result<&'schema PlatformMapper, PluginError> {
    let os = env.os.to_string();
    let mut platform = schema.platform.get(&os);

    // Fallback to linux for other OSes
    if platform.is_none() && env.os.is_bsd() {
        platform = schema.platform.get("linux");
    }

    platform.ok_or_else(|| PluginError::UnsupportedOS {
        tool: schema.name.clone(),
        os,
    })
}

fn get_bin_path(platform: &PlatformMapper, env: &HostEnvironment) -> Result<String, Error> {
    let id = get_tool_id()?;

    Ok(platform
        .bin_path
        .clone()
        .unwrap_or_else(|| env.os.get_exe_name(id)))
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
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        self_upgrade_commands: schema.metadata.self_upgrade_commands,
        ..ToolMetadataOutput::default()
    }))
}

pub fn remove_v_prefix(value: &str) -> &str {
    if value.starts_with('v') || value.starts_with('V') {
        return &value[1..];
    }

    value
}

fn create_version(cap: Captures) -> String {
    // If no named, use entire string (legacy)
    if cap.name("major").is_none() {
        return cap.get(1).unwrap().as_str().to_string();
    }

    // Otherwise piece named parts together
    let mut version = String::new();

    version.push_str(cap.name("major").map(|c| c.as_str()).unwrap_or("0"));
    version.push('.');
    version.push_str(cap.name("minor").map(|c| c.as_str()).unwrap_or("0"));
    version.push('.');
    version.push_str(cap.name("patch").map(|c| c.as_str()).unwrap_or("0"));

    if let Some(pre) = cap.name("pre").map(|c| c.as_str()) {
        if !pre.starts_with('-') {
            version.push('-');
        }
        version.push_str(pre);
    }

    if let Some(build) = cap.name("build").map(|c| c.as_str()) {
        if !build.starts_with('+') {
            version.push('+');
        }
        version.push_str(build);
    }

    version
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let schema = get_schema()?;

    if let Some(repository) = schema.resolve.git_url {
        let pattern = regex::Regex::new(
            schema
                .resolve
                .git_tag_pattern
                .as_ref()
                .unwrap_or(&schema.resolve.version_pattern),
        )?;

        let tags = load_git_tags(repository)?;
        let tags = tags
            .into_iter()
            .filter_map(|t| pattern.captures(&t).map(create_version))
            .collect::<Vec<_>>();

        return Ok(Json(LoadVersionsOutput::from(tags)?));
    }

    if let Some(endpoint) = schema.resolve.manifest_url {
        let pattern = regex::Regex::new(&schema.resolve.version_pattern)?;
        let version_key = &schema.resolve.manifest_version_key;
        let response: Vec<JsonValue> = fetch_url(endpoint)?;
        let mut versions = vec![];

        let mut push_version = |v: &str| {
            if let Some(cap) = pattern.captures(v) {
                versions.push(create_version(cap));
            }
        };

        for row in response {
            match row {
                JsonValue::String(v) => {
                    push_version(&v);
                }
                JsonValue::Object(o) => {
                    if let Some(JsonValue::String(v)) = o.get(version_key) {
                        push_version(v);
                    }
                }
                _ => {}
            }
        }

        return Ok(Json(LoadVersionsOutput::from(versions)?));
    }

    err!("Unable to resolve versions for {}. Schema either requires a `git_url` or `manifest_url`.")
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

fn interpolate_tokens(
    value: &str,
    version: &str,
    schema: &Schema,
    env: &HostEnvironment,
) -> String {
    let arch = env.arch.to_rust_arch();
    let os = env.os.to_string();

    let mut value = value
        .replace("{version}", version)
        .replace("{arch}", schema.install.arch.get(&arch).unwrap_or(&arch))
        .replace("{os}", &os);

    // Avoid detecting musl unless requested
    if value.contains("{libc}") {
        value = value.replace(
            "{libc}",
            if env.os != HostOS::MacOS && env.os != HostOS::Windows && is_musl(env) {
                "musl"
            } else {
                "gnu"
            },
        );
    }

    value
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_proto_environment()?;
    let schema = get_schema()?;
    let platform = get_platform(&schema, &env)?;
    let version = input.context.version.to_string();
    let is_canary = version == "canary";

    let download_file = interpolate_tokens(&platform.download_file, &version, &schema, &env);

    let download_url = interpolate_tokens(
        if is_canary {
            schema
                .install
                .download_url_canary
                .as_ref()
                .unwrap_or(&schema.install.download_url)
        } else {
            &schema.install.download_url
        },
        &version,
        &schema,
        &env,
    )
    .replace("{download_file}", &download_file);

    let checksum_file = interpolate_tokens(
        platform.checksum_file.as_deref().unwrap_or("CHECKSUM.txt"),
        &version,
        &schema,
        &env,
    );

    let checksum_url = if is_canary {
        schema
            .install
            .checksum_url_canary
            .as_ref()
            .or(schema.install.checksum_url.as_ref())
    } else {
        schema.install.checksum_url.as_ref()
    };

    let checksum_url = checksum_url.map(|url| {
        interpolate_tokens(url, &version, &schema, &env).replace("{checksum_file}", &checksum_file)
    });

    let archive_prefix = platform
        .archive_prefix
        .as_ref()
        .map(|prefix| interpolate_tokens(prefix, &version, &schema, &env));

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix,
        checksum_url,
        checksum_name: Some(checksum_file),
        checksum_public_key: schema.install.checksum_public_key,
        download_url,
        download_name: Some(download_file),
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_proto_environment()?;
    let schema = get_schema()?;
    let platform = get_platform(&schema, &env)?;

    let mut primary = ExecutableConfig::new(get_bin_path(platform, &env)?);
    primary.no_bin = schema.install.no_bin;
    primary.no_shim = schema.install.no_shim;

    Ok(Json(LocateExecutablesOutput {
        globals_lookup_dirs: schema.globals.lookup_dirs,
        globals_prefix: schema.globals.package_prefix,
        primary: Some(primary),
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn install_global(
    Json(input): Json<InstallGlobalInput>,
) -> FnResult<Json<InstallGlobalOutput>> {
    let schema = get_schema()?;

    if let Some(install_args) = schema.globals.install_args {
        let args = install_args
            .into_iter()
            .map(|arg| arg.replace("{dependency}", &input.dependency))
            .collect::<Vec<_>>();

        let result = exec_command!(inherit, get_tool_id()?, args);

        return Ok(Json(InstallGlobalOutput::from_exec_command(result)));
    }

    Ok(Json(InstallGlobalOutput::default()))
}

#[plugin_fn]
pub fn uninstall_global(
    Json(input): Json<UninstallGlobalInput>,
) -> FnResult<Json<UninstallGlobalOutput>> {
    let schema = get_schema()?;

    if let Some(uninstall_args) = schema.globals.uninstall_args {
        let args = uninstall_args
            .into_iter()
            .map(|arg| arg.replace("{dependency}", &input.dependency))
            .collect::<Vec<_>>();

        let result = exec_command!(inherit, get_tool_id()?, args);

        return Ok(Json(UninstallGlobalOutput::from_exec_command(result)));
    }

    Ok(Json(UninstallGlobalOutput::default()))
}
