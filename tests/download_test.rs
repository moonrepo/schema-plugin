use proto_pdk_test_utils::*;
use starbase_sandbox::{create_empty_sandbox, locate_fixture};

generate_download_install_tests!(
    "schema-test",
    "1.10.0",
    Some(locate_fixture("schemas").join("base.toml"))
);

#[test]
fn supports_linux_arm64() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::Arm64,
        os: HostOS::Linux,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: Some("moon-linux-aarch64-20.0.0".into()),
            checksum_name: Some("CHECKSUM.txt".into()),
            download_name: Some("moon-aarch64-unknown-linux-gnu".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-aarch64-unknown-linux-gnu".into(),
            ..Default::default()
        }
    );
}

#[test]
fn supports_linux_x64() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X64,
        os: HostOS::Linux,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: Some("moon-linux-x86_64-20.0.0".into()),
            checksum_name: Some("CHECKSUM.txt".into()),
            download_name: Some("moon-x86_64-unknown-linux-gnu".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-x86_64-unknown-linux-gnu".into(),
            ..Default::default()
        }
    );
}

#[test]
fn supports_macos_arm64() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::Arm64,
        os: HostOS::MacOS,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: None,
            checksum_name: Some("SHASUM256.txt".into()),
            download_name: Some("moon-aarch64-apple-darwin".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-aarch64-apple-darwin".into(),
            ..Default::default()
        }
    );
}

#[test]
fn supports_macos_x64() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X64,
        os: HostOS::MacOS,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: None,
            checksum_name: Some("SHASUM256.txt".into()),
            download_name: Some("moon-x86_64-apple-darwin".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-x86_64-apple-darwin".into(),
            ..Default::default()
        }
    );
}

#[test]
fn supports_windows_arm64() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::Arm64,
        os: HostOS::Windows,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: None,
            checksum_name: Some("CHECKSUM.txt".into()),
            download_name: Some("moon-aarch64-pc-windows-msvc.exe".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-aarch64-pc-windows-msvc.exe".into(),
            ..Default::default()
        }
    );
}

#[test]
fn supports_windows_x86() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X86,
        os: HostOS::Windows,
        ..Default::default()
    });

    assert_eq!(
        plugin.download_prebuilt(DownloadPrebuiltInput {
            context: ToolContext {
                version: VersionSpec::parse("20.0.0").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        }),
        DownloadPrebuiltOutput {
            archive_prefix: None,
            checksum_name: Some("CHECKSUM.txt".into()),
            download_name: Some("moon-x86-pc-windows-msvc.exe".into()),
            download_url: "https://github.com/moonrepo/moon/releases/download/v20.0.0/moon-x86-pc-windows-msvc.exe".into(),
            ..Default::default()
        }
    );
}

#[test]
fn locates_linux_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::Arm64,
        os: HostOS::Linux,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: VersionSpec::parse("20.0.0").unwrap(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("lin/moon".into())
    );
}

#[test]
fn locates_macos_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X64,
        os: HostOS::MacOS,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: VersionSpec::parse("20.0.0").unwrap(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("mac/moon".into())
    );
}

#[test]
fn locates_windows_bin() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas").join("bins.toml"),
    );

    plugin.set_environment(HostEnvironment {
        arch: HostArch::X64,
        os: HostOS::Windows,
        ..Default::default()
    });

    assert_eq!(
        plugin
            .locate_bins(LocateBinsInput {
                context: ToolContext {
                    version: VersionSpec::parse("20.0.0").unwrap(),
                    ..Default::default()
                },
            })
            .bin_path,
        Some("win/moon.exe".into())
    );
}
