fn main() -> std::io::Result<()> {
    // esphome proto files from release 2025.12.2
    prost_build::compile_protos(&["esphome_2025.12.2/api.proto"], &["esphome_2025.12.2/"])?;
    Ok(())
}