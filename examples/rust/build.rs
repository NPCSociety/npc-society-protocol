//! Build script to compile proto files.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto files using tonic-build
    // Only build server since this is the daemon example
    tonic_build::configure()
        .build_server(true)
        .build_client(false)  // Don't build client to avoid method name collision
        .compile_protos(
            &["../../proto/npc_society/v1/npc_society.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
