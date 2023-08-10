use proto_pdk_test_utils::*;
use starbase_sandbox::{create_empty_sandbox, locate_fixture};
use std::path::PathBuf;

generate_download_install_tests!(
    "schema-test",
    "1.10.0",
    Some(locate_fixture("schemas").join("base.toml"))
);
