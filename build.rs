use std::collections::HashSet;
use std::fs::{read_dir,create_dir_all,read_to_string,write,DirEntry};
use std::path::{Path,PathBuf};

use shaderc::ShaderKind;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR environment variable.");
    let shaders_dir = Path::new(&manifest_dir).join("resources/shaders");
    let output_directory = Path::new(&manifest_dir).join("out");

    let mut shaderc_compiler = shaderc::Compiler::new().expect("Failed to create shaderc compiler.");

    let mut made_directories: HashSet<PathBuf> = HashSet::new();

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
            let shader_source = read_to_string(&shader_path_buf).expect("Failed to read shader file.");

            let shader_file_name = &shader_path_buf.file_name().expect("Failed to get shader file name.")
                .to_str().expect("Failed to get shader file name as string.");

            let shader_file_stem = &shader_path_buf.file_stem().expect("Failed to get shader file stem.")
                .to_str().expect("Failed to get shader file stem as string.");

            let shader_file_stem_prefix: String = shader_file_stem.chars().take_while(|x| x != &'_').collect();

            let shader_kind = match shader_file_stem_prefix.as_str() {
                "vertex" => ShaderKind::DefaultVertex,
                "fragment" => ShaderKind::DefaultFragment,
                "compute" => ShaderKind::DefaultCompute,
                "geometry" => ShaderKind::DefaultGeometry,
                "tessctrl" => ShaderKind::DefaultTessControl,
                "tesseval" => ShaderKind::DefaultTessEvaluation,
                "raygen" => ShaderKind::DefaultRayGeneration,
                "anyhit" => ShaderKind::DefaultAnyHit,
                "closesthit" => ShaderKind::DefaultClosestHit,
                "miss" => ShaderKind::DefaultMiss,
                "intersection" => ShaderKind::Intersection,
                "callable" => ShaderKind::DefaultCallable,
                "task" => ShaderKind::DefaultTask,
                "mesh" => ShaderKind::DefaultMesh,
                _ => ShaderKind::InferFromSource,
            };

            let shader_artifact = shaderc_compiler.compile_into_spirv(
                shader_source.as_str(),
                shader_kind,
                &shader_file_name,
                "main",
                None
            ).expect("Failed to compile shader.");

            let shader_binary = shader_artifact.as_binary_u8();

            let relative_path = &shader_path_buf.strip_prefix(&manifest_dir).expect("Failed to get shader's relative path.");
            let relative_path = relative_path.with_extension("spv");

            let adjusted_path = Path::new(&output_directory).join(relative_path);
            let adjusted_path_directory = adjusted_path.parent().expect("Failed to get directory of SPIR-V output path.");

            if made_directories.get(adjusted_path_directory).is_none() {
                create_dir_all(adjusted_path_directory).expect("Failed to create directory of SPIR-V output.");
                made_directories.insert(adjusted_path_directory.clone().to_path_buf());
            }

            write(&adjusted_path, shader_binary).expect("Failed to write shader.");

            println!("cargo:rerun-if-changed={}", &adjusted_path.as_os_str().to_str().unwrap());
            println!("cargo:rerun-if-changed={}", &adjusted_path_directory.as_os_str().to_str().unwrap());
        }
    }

    println!("cargo:rerun-if-changed={}", &output_directory.as_os_str().to_str().unwrap());
}