use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

fn main() {
    // The proto files defining the message types we want to support.
    let roots = ["protos/perfetto/trace/trace.proto"];
    let protoc = &protoc_bin_vendored::protoc_bin_path().unwrap();

    let (descriptor_set_out_path, deps_raw) = dep_list(protoc, &roots);
    let files = parse_dep_list(&descriptor_set_out_path.to_string_lossy(), &deps_raw);

    // Generate Rust code from protos.
    protobuf_codegen::Codegen::new()
        .protoc()
        .protoc_path(protoc)
        .include(".")
        .inputs(files)
        .cargo_out_dir("protos")
        .run_from_script();
}

fn parse_dep_list<'a>(
    descriptor_set_out: &str,
    dep_list: &'a str,
) -> impl Iterator<Item = &'a str> {
    dep_list
        .strip_prefix(&format!("{}: ", descriptor_set_out))
        .unwrap()
        .split("\\\n ")
}

#[cfg(not(windows))]
fn dep_list(protoc: &PathBuf, roots: &[&str]) -> (PathBuf, String) {
    let child = Command::new(protoc)
        .arg("--dependency_out=/dev/stdout")
        .arg("--descriptor_set_out=/dev/null")
        .args(roots)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    (
        PathBuf::from("/dev/null"),
        String::from_utf8(result.stdout).unwrap(),
    )
}

#[cfg(windows)]
fn dep_list(protoc: &PathBuf, roots: &[&str]) -> (PathBuf, String) {
    let mut dependency_out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    dependency_out.push("dependency_out");

    // NUL does not work on windows, and file has to be relative to project root otherwise...
    let (_, descriptor_set_out_path) = tempfile::NamedTempFile::new_in(".")
        .unwrap()
        .keep()
        .unwrap();

    struct DeleteOnDrop(PathBuf);
    impl Drop for DeleteOnDrop {
        fn drop(&mut self) {
            std::fs::remove_file(&self.0).unwrap()
        }
    }
    let descriptor_set_out = DeleteOnDrop(descriptor_set_out_path.clone());

    let mut child = Command::new(protoc)
        .arg(format!("--dependency_out={}", dependency_out.display()))
        .arg(format!(
            "--descriptor_set_out={}",
            descriptor_set_out.0.display()
        ))
        .args(roots)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let result = child.wait().unwrap();
    drop(descriptor_set_out);
    assert!(result.success());

    (
        descriptor_set_out_path,
        std::fs::read_to_string(dependency_out).unwrap(),
    )
}
