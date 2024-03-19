use crate::schema::{ExecutableSchema, PlatformMapper, Schema, SchemaType};
use extism_pdk::*;
use proto_pdk::*;
use regex::Captures;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

fn get_schema() -> Result<Schema, Error> {
    let data = config::get("proto_schema")?.expect("Missing schema!");
    let schema: Schema = json::from_str(&data)?;

    Ok(schema)
}

fn get_platform<'schema>(
    schema: &'schema Schema,
    env: &HostEnvironment,
) -> Result<&'schema PlatformMapper, PluginError> {
    let mut platform = schema.platform.get(&env.os);

    // Fallback to linux for other OSes
    if platform.is_none() && env.os.is_bsd() {
        platform = schema.platform.get(&HostOS::Linux);
    }

    platform.ok_or_else(|| PluginError::UnsupportedOS {
        tool: schema.name.clone(),
        os: env.os.to_rust_os(),
    })
}

fn get_platform_exe_path(
    schema: &Schema,
    env: &HostEnvironment,
    platform: &PlatformMapper,
    version: &str,
) -> Result<String, Error> {
    let id = get_plugin_id()?;

    Ok(platform
        .bin_path
        .clone()
        .map(|s| interpolate_tokens(&s, version, schema, env))
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

    Err(plugin_err!(
        "Unable to resolve versions for {}. Schema either requires a <property>resolve.git_url</property> or <property>resolve.manifest_url</property>.",
        schema.name
    ))
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
        .replace(
            "{arch}",
            schema.install.arch.get(&env.arch).unwrap_or(&arch),
        )
        .replace("{os}", &os);

    // Avoid detecting musl unless requested
    if value.contains("{libc}") {
        let libc = env.libc.to_string();

        value = value.replace(
            "{libc}",
            schema.install.libc.get(&env.libc).unwrap_or(&libc),
        );
    }

    value
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;
    let schema = get_schema()?;
    let platform = get_platform(&schema, &env)?;

    if !platform.archs.is_empty() {
        check_supported_os_and_arch(
            &schema.name,
            &env,
            HashMap::from_iter([(env.os, platform.archs.clone())]),
        )?;
    }

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

fn create_executable_config(schema: ExecutableSchema) -> ExecutableConfig {
    ExecutableConfig {
        exe_path: schema.exe_path,
        exe_link_path: schema.exe_link_path,
        no_bin: schema.no_bin,
        no_shim: schema.no_shim,
        parent_exe_name: schema.parent_exe_name,
        shim_before_args: schema.shim_before_args.map(|args| StringOrVec::Vec(args)),
        shim_after_args: schema.shim_after_args.map(|args| StringOrVec::Vec(args)),
        shim_env_vars: schema.shim_env_vars.map(|env| HashMap::from_iter(env)),
    }
}

#[plugin_fn]
pub fn locate_executables(
    Json(input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let schema = get_schema()?;
    let platform = get_platform(&schema, &env)?;
    let version = input.context.version.to_string();

    // On Windows, automatically add the `.exe` extension to all executables.
    // But only if there is no extension, so that we don't overwrite `.js` and others!
    let append_exe_ext = |mut path: PathBuf| -> PathBuf {
        if env.os.is_windows() && path.extension().is_none() {
            path.set_extension("exe");
        }

        path
    };

    // Primary exe
    let mut primary = schema
        .install
        .primary
        .clone()
        .map(create_executable_config)
        .unwrap_or_default();

    if let Some(exe_path) = primary.exe_path.take() {
        primary.exe_path = Some(append_exe_ext(exe_path));
    } else {
        primary.exe_path = Some(get_platform_exe_path(&schema, &env, platform, &version)?.into());
    }

    if let Some(no_bin) = schema.install.no_bin {
        primary.no_bin = no_bin;
    }

    if let Some(no_shim) = schema.install.no_shim {
        primary.no_shim = no_shim;
    }

    // Secondary exe's
    let secondary = schema.install.secondary.into_iter().map(|(key, value)| {
        let mut config = create_executable_config(value);

        if let Some(exe_path) = config.exe_path.take() {
            config.exe_path = Some(append_exe_ext(exe_path));
        }

        if let Some(exe_link_path) = config.exe_link_path.take() {
            config.exe_link_path = Some(append_exe_ext(exe_link_path));
        }

        (key, config)
    });

    Ok(Json(LocateExecutablesOutput {
        globals_lookup_dirs: schema.packages.globals_lookup_dirs,
        globals_prefix: schema.packages.globals_prefix,
        primary: Some(primary),
        secondary: HashMap::from_iter(secondary),
    }))
}
