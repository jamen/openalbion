use std::fs::{read_dir,read_to_string,write,DirEntry};
use std::path::Path;

use shaderc::ShaderKind;

pub fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR environment variable.");
    let shaders_dir = Path::new(&manifest_dir).join("src");

    let mut shaderc_compiler = shaderc::Compiler::new().expect("Failed to create shaderc compiler.");

    let mut entries: Vec<DirEntry> = read_dir(&shaders_dir)
        .expect("Failed to read GLSL input directory.")
        .map(|x| x.expect("Failed to get item in GLSL input directory."))
        .into_iter()
        .collect();

    loop {
        let entry = match entries.pop() {
            Some(entry) => entry,
            None => break
        };

        let shader_path_buf = entry.path();

        println!("cargo:rerun-if-changed={}", shader_path_buf.as_os_str().to_str().unwrap());

        let metadata = entry.metadata().expect("Failed to get metadata of item in GLSL input directory.");

        if metadata.is_dir() {
            entries.extend(
                read_dir(&shader_path_buf)
                    .expect("Failed to read subdirectory item in GLSL input directory.")
                    .map(|x| x.expect("Failed to get subdirectory item in GLSL input directory."))
                    .into_iter()
            );
        } else if metadata.is_file() {
            let ext = shader_path_buf.extension().map(|x| x.to_str().unwrap());

            let shader_kind = match ext {
                Some("vert") => ShaderKind::DefaultVertex,
                Some("frag") => ShaderKind::DefaultFragment,
                Some("comp") => ShaderKind::DefaultCompute,
                Some("geometry") => ShaderKind::DefaultGeometry,
                Some("tessctrl") => ShaderKind::DefaultTessControl,
                Some("tesseval") => ShaderKind::DefaultTessEvaluation,
                Some("raygen") => ShaderKind::DefaultRayGeneration,
                Some("anyhit") => ShaderKind::DefaultAnyHit,
                Some("closesthit") => ShaderKind::DefaultClosestHit,
                Some("miss") => ShaderKind::DefaultMiss,
                Some("intersection") => ShaderKind::Intersection,
                Some("callable") => ShaderKind::DefaultCallable,
                Some("task") => ShaderKind::DefaultTask,
                Some("mesh") => ShaderKind::DefaultMesh,
                Some("glsl") => ShaderKind::InferFromSource,
                _ => continue
            };

            let shader_file_name = &shader_path_buf.file_name().unwrap().to_str().unwrap();

            let shader_source = read_to_string(&shader_path_buf).unwrap();

            let shader_artifact = shaderc_compiler.compile_into_spirv(
                shader_source.as_str(),
                shader_kind,
                &shader_file_name,
                "main",
                None
            ).expect("Failed to compile shader.");

            let shader_binary = shader_artifact.as_binary_u8();

            let output_path = shader_path_buf.with_extension(format!("{}.spv", ext.unwrap()));

            write(&output_path, shader_binary).expect("Failed to write shader.");

            println!("cargo:rerun-if-changed={}", &output_path.as_os_str().to_str().unwrap());
        }
    }

    println!("cargo:rerun-if-changed={}", &shaders_dir.as_os_str().to_str().unwrap());
}