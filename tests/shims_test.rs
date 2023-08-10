use proto_pdk_test_utils::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox, locate_fixture};

#[cfg(not(windows))]
generate_global_shims_test!(
    "schema-test",
    [],
    Some(locate_fixture("schemas").join("base.toml"))
);

#[tokio::test]
async fn doesnt_create_global_shim() {
    let sandbox = create_empty_sandbox();
    let plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas/shim-no-global.toml"),
    );

    plugin.tool.create_shims(false).await.unwrap();

    assert!(!sandbox.path().join(".proto/bin/schema-test").exists());
}

#[tokio::test]
async fn can_create_local_shim() {
    let sandbox = create_empty_sandbox();
    let plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas/shim-local.toml"),
    );

    plugin.tool.create_shims(false).await.unwrap();

    assert!(sandbox
        .path()
        .join(".proto/tools/schemas/latest/shims/schemas")
        .exists());

    assert_snapshot!(std::fs::read_to_string(
        sandbox
            .path()
            .join(".proto/tools/schemas/latest/shims/schemas")
    )
    .unwrap());
}
