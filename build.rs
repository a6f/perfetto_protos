use std::process::Command;
use std::process::Stdio;

fn main() {
    // The proto files defining the message types we want to support.
    let roots = ["protos/perfetto/trace/trace.proto"];
    let protoc = &protoc_bin_vendored::protoc_bin_path().unwrap();

    // Find the transitive deps of `roots`.
    let child = Command::new(protoc)
        .arg("--dependency_out=/dev/stdout")
        .arg("--descriptor_set_out=/dev/null")
        .args(roots)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    let output = core::str::from_utf8(&result.stdout).unwrap();
    let output = output.replace("\\\n", " ");
    let files: Vec<&str> = output.split_ascii_whitespace().collect();

    // Generate Rust code from protos.
    protobuf_codegen::Codegen::new()
        .protoc()
        .protoc_path(protoc)
        .include(".")
        .inputs(&files)
        .cargo_out_dir("protos")
        .run_from_script();
}
