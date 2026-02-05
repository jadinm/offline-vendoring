mod common;

use rstest::rstest;
use tracing::info;

#[rstest]
#[case("World")]
#[case("Rust")]
#[test_log::test]
fn hello(#[case] name: &str) {
    info!("Hello, {name}");
}
