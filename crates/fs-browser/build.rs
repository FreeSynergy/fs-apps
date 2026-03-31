fn main() {
    let proto_files = ["proto/browser.proto"];
    let include_dirs = ["proto"];

    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&proto_files, &include_dirs)
        .unwrap_or_else(|e| panic!("tonic_build failed: {e}"));
}
