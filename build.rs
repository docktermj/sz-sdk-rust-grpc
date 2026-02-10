fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/szconfig.proto")?;
    tonic_build::compile_protos("proto/szconfigmanager.proto")?;
    tonic_build::compile_protos("proto/szdiagnostic.proto")?;
    tonic_build::compile_protos("proto/szengine.proto")?;
    tonic_build::compile_protos("proto/szproduct.proto")?;
    Ok(())
}
