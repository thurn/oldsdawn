// Proto compilation requires that the $PROTOC and $PROTOC_INCLUDE
// environment variables be set. For example if protoc is installed via
// Homebrew for OSX, this might mean:
//
//   - PROTOC="/opt/homebrew/bin/protoc"
//   - PROTOC_INCLUDE="/opt/homebrew/include"

use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building protocol buffers");
    tonic_build::configure()
        .build_client(false)
        .out_dir("src/generated")
        .compile(&["proto/color.proto"], &["proto/"])?;
    Ok(())
}
