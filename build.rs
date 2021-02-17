fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/factory.proto"
    ];

    for proto in protos.iter() {
        tonic_build::compile_protos(proto).unwrap()
    }

    Ok(())
}