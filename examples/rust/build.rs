//! Build script to compile proto files.
//! 
//! In production, you would use the pre-generated code from `buf generate`.
//! This build script is here for standalone development/testing.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if generated code already exists from buf
    let gen_path = std::path::Path::new("../../gen/rust/src");
    if gen_path.exists() {
        println!("cargo:warning=Using pre-generated code from buf generate");
        return Ok(());
    }

    // Otherwise, compile proto files directly
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../../proto/npc_society/v1/npc_society.proto"],
            &["../../proto"],
        )?;

    Ok(())
}

