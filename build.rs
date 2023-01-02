extern crate prost_build;

fn main() {
    prost_build::compile_protos(
        &[
            "proto/hex_pb_names.proto",
            "proto/hex_pb_package.proto",
            "proto/hex_pb_signed.proto",
            "proto/hex_pb_versions.proto",
        ],
        &["proto/"],
    )
    .unwrap();
}
