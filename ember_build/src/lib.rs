use std::collections::HashSet;
use std::fs::{read_dir,create_dir_all,read_to_string,write,DirEntry};
use std::path::{Path,PathBuf};

use shaderc::ShaderKind;

pub struct EmberBuild {
    pub shaders: PathBuf,
}

pub fn build(config: EmberBuild) {
    let output_directory = std::env::var("OUT_DIR").expect("Failed to get OUT_DIR environment variable.");

    let mut shaderc_compiler = shaderc::Compiler::new().expect("Failed to create shaderc compiler.");

    let mut made_directories: HashSet<PathBuf> = HashSet::new();

    let mut entries: Vec<DirEntry> = read_dir(&config.shaders)
        .expect("Failed to read GLSL input directory.")
        .map(|x| x.expect("Failed to get item in GLSL input directory."))
        .into_iter()
        .collect();

    loop {
        let entry = match entries.pop() {
            Some(entry) => entry,
            None => return
        };

        let path_buf = entry.path();
        let metadata = entry.metadata().expect("Failed to get metadata of item in GLSL input directory.");

        let path_str = path_buf.as_os_str().to_str().expect("Failed to get GLSL input path as string.");

        println!("cargo:rerun-if-changed={}", path_str);

        if metadata.is_dir() {
            entries.extend(
                read_dir(&path_buf)
                    .expect("Failed to read subdirectory item in GLSL input directory.")
                    .map(|x| x.expect("Failed to get subdirectory item in GLSL input directory."))
                    .into_iter()
            );
        } else if metadata.is_file() {
            let shader_source = read_to_string(&path_buf).expect("Failed to read shader file.");
            let shader_file_name = &path_buf.file_name().expect("Failed to get shader file name.")
                .to_str().expect("Failed to get shader file name as string.");

            let shader_file_stem = &path_buf.file_stem().expect("Failed to get shader file stem.")
                .to_str().expect("Failed to get shader file stem as string.");

            let shader_file_prefix_type: String = shader_file_stem.chars().take_while(|x| x != &'_').collect();

            let shader_kind = match shader_file_prefix_type.as_str() {
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

            let relative_path = &path_buf.strip_prefix(&config.shaders).expect("Failed to get shader's relative path.");
            let relative_path = relative_path.with_extension("spv");

            let adjusted_path = Path::new(&output_directory).join(relative_path);
            let adjusted_path_directory = adjusted_path.parent().expect("Failed to get directory of SPIR-V output path.");

            if made_directories.get(adjusted_path_directory).is_none() {
                create_dir_all(adjusted_path_directory).expect("Failed to create directory of SPIR-V output.");
                made_directories.insert(adjusted_path.clone());
            }

            write(adjusted_path, shader_binary).expect("Failed to write shader.");
        }
    }
}