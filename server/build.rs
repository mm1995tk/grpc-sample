fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/sample_a.proto")?;
    tonic_build::compile_protos("../proto/sample_b.proto")?;
    Ok(())
}
