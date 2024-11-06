use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
};

use fs_err as fs;
use keelhaul::{ArchWidth, CodegenConfig, Filter, Filters, ListFilter, ModelSource, SourceFormat};

/// Open a file handle to the final output file
fn write_file(path: impl AsRef<Path>, text: &str) {
    let path = path.as_ref();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Failed to open output file.");
    f.write_all(text.as_bytes()).unwrap();
}

fn main() {
    let in_file = ModelSource::new(
        "../../../svd/headsail-hpc-v0.1.1.svd".into(),
        SourceFormat::Svd(keelhaul::ValidateLevel::Strict),
    );
    let test_cfg = CodegenConfig::default()
        .on_fail(keelhaul::FailureImplKind::ReturnError)
        .derive_debug(true);
    let device = env::var("DEVICE").ok().map(|d| vec![d]);
    let filters = Filters::from_filters(
        None,
        Some(Box::new(ListFilter::new(device, vec![])) as Box<dyn Filter>),
        None,
    );
    let text = keelhaul::generate_tests_with_format(
        &[in_file],
        ArchWidth::U64,
        &test_cfg,
        &filters,
        false,
        false,
    )
    .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("tests.rs");
    write_file(out_path, &text);
}
