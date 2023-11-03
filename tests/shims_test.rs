use proto_pdk_test_utils::*;
use starbase_sandbox::{create_empty_sandbox, locate_fixture};

#[cfg(not(windows))]
generate_shims_test!(
    "schema-test",
    [],
    Some(locate_fixture("schemas").join("base.toml"))
);

#[tokio::test]
async fn doesnt_create_global_shim() {
    let sandbox = create_empty_sandbox();
    let mut plugin = create_schema_plugin(
        "schema-test",
        sandbox.path(),
        locate_fixture("schemas/shim-no-global.toml"),
    );

    plugin.tool.generate_shims(false).await.unwrap();

    assert!(!sandbox.path().join(".proto/bin/schema-test").exists());
}
