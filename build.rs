use glob::glob;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let protos: Vec<PathBuf> = glob("api/envoy/api/envoy/**/v3/*.proto")
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    let mut config = prost_build::Config::new();
    config.disable_comments(["."]);
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_well_known_types(true)
        .include_file("mod.rs")
        // .skip_protoc_run()
        .compile_with_config(
            config,
            &protos,
            &[
                "api/envoy/api",
                "api/googleapis",
                "api/protoc-gen-validate",
                "api/xds",
                "api/opencensus-proto/src",
                "api/opentelemetry-proto",
                "api/client_model",
            ],
        )?;
    Ok(())
}