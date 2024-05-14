fn main() {
    let compile_result = tonic_build::compile_protos("proto/match-engine.proto");

    match compile_result {
        Ok(_) => {},
        Err(err) => eprintln!("Compile proto files failed due to: {}", err)
    }
}