fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .include_file("mod.rs")
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "../proto/hospitalization.proto",
                "../proto/newcase.proto",
                "../proto/mix.proto",
                "../proto/icu.proto",
            ], 
            &["../proto"]
        )?;

    Ok(())
}
