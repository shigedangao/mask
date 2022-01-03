fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/hospitalization.proto")?;
    tonic_build::compile_protos("../proto/newcase.proto")?;
    tonic_build::compile_protos("../proto/mix.proto")?;

    Ok(())
}
