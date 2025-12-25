use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static HELLO_BINARY: OnceLock<PathBuf> = OnceLock::new();

pub fn hello_fixture_path() -> PathBuf {
    HELLO_BINARY
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let manifest = root.join("tests/fixtures/Cargo.toml");
            let target_dir = root.join("target/fixtures");

            let status = Command::new("cargo")
                .args([
                    "build",
                    "--manifest-path",
                    manifest
                        .to_str()
                        .expect("fixture manifest path should be valid UTF-8"),
                    "--bin",
                    "jdb-inferior-fixtures",
                ])
                .env("CARGO_TARGET_DIR", &target_dir)
                .status()
                .expect("failed to run cargo to build fixture");

            assert!(
                status.success(),
                "building inferior fixture failed: {status:?}"
            );

            target_dir.join("debug/jdb-inferior-fixtures")
        })
        .clone()
}
