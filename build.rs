fn main() {
    build_spirv_kernel();
}

fn build_spirv_kernel() {
    use spirv_builder::SpirvBuilder;
    use std::path::PathBuf;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_path = PathBuf::from(manifest_dir).join("kernel");

    let result = SpirvBuilder::new(crate_path, "spirv-unknown-spv1.6")
        .print_metadata(spirv_builder::MetadataPrintout::Full)
        .build()
        .unwrap();

    // Export the kernel path for the runtime to use
    println!(
        "cargo:rustc-env=KERNEL_SPV_PATH={}",
        result.module.unwrap_single().display()
    );
}
