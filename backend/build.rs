fn main() {
    tonic_build::configure()
        .compile(&["proto/messaging.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile proto files: {}", e));
}