use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::path::Path;

fn main() {
    let files = ["resource/airports.csv", "resource/runways.csv"];

    let mut hasher = DefaultHasher::new();
    for f in files {
        let path = Path::new(f);
        let bytes = std::fs::read(path)
            .unwrap_or_else(|e| panic!("build.rs: failed to read {}: {e}", path.display()));
        hasher.write(&(bytes.len() as u64).to_le_bytes());
        hasher.write(&bytes);
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("data_hash.rs");
    std::fs::write(
        &dest,
        format!("pub const DATA_HASH: u64 = {};\n", hasher.finish()),
    )
    .unwrap_or_else(|e| panic!("build.rs: failed to write {}: {e}", dest.display()));
}
