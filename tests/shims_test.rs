use proto_pdk_test_utils::*;
use starbase_sandbox::locate_fixture;

#[cfg(not(windows))]
generate_shims_test!(
    "schema-test",
    [],
    Some(locate_fixture("schemas").join("base.toml"))
);

#[tokio::test]
async fn doesnt_create_global_shim() {
    let sandbox = create_empty_proto_sandbox();
    let mut plugin =
        sandbox.create_schema_plugin("schema-test", locate_fixture("schemas/shim-no-global.toml"));

    plugin.tool.generate_shims(false).await.unwrap();

    assert!(!sandbox.proto_dir.join("bin/schema-test").exists());
}
