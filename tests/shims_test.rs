use proto_pdk_test_utils::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox, locate_fixture};

#[cfg(not(windows))]
generate_global_shims_test!(
    "schema-test",
    [],
    Some(locate_fixture("schemas").join("base.toml"))
);
