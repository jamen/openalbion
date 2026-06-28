use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::mesh::Mesh;
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct MeshInfoArgs {
    /// A .big archive (e.g. data/graphics/graphics.big)
    input: PathBuf,

    /// The mesh asset's symbol name (e.g. MESH_OBJECT_CHEST_01)
    asset: String,
}

pub fn handler(args: MeshInfoArgs) -> anyhow::Result<()> {
    let file = File::open(&args.input).context("Could not open big file")?;
    let mut reader = BigReader::new(file).context("Could not read big file")?;

    // Find the asset's bank by symbol name.
    let bank_name = reader
        .bank_iter()
        .find(|bank| bank.asset(&args.asset).is_some())
        .map(|bank| bank.metadata().name.to_string())
        .ok_or_else(|| anyhow!("No asset named {:?} in {:?}", args.asset, args.input))?;

    let (metadata, bytes) = reader
        .read_asset(&bank_name, &args.asset)
        .context("Could not read asset")?;

    let mesh = Mesh::decode(&bytes).context("Could not decode mesh")?;

    println!("{} ({}/{})", mesh.name, bank_name, args.asset);
    println!("  animated         {}", mesh.animated);
    println!(
        "  bounding box     min {:?} max {:?}",
        mesh.bounding_box.min, mesh.bounding_box.max
    );
    println!(
        "  bounding sphere  centre {:?} radius {:.3}",
        mesh.bounding_sphere.center, mesh.bounding_sphere.radius
    );
    println!("  bones            {}", mesh.bones.len());
    println!("  materials        {}", mesh.materials.len());
    for (idx, material) in mesh.materials.iter().enumerate() {
        println!(
            "    [{idx}] {:?} base_texture={} ({})",
            material.name,
            material.base_texture_id,
            if material.transparent {
                "transparent"
            } else {
                "opaque"
            }
        );
    }
    if let Some(ExtraMetadata::Mesh(m)) = &metadata.extras {
        println!("  texture ids      {:?}", m.texture_ids);
    }
    println!("  primitives       {}", mesh.primitives.len());
    for (idx, prim) in mesh.primitives.iter().enumerate() {
        println!(
            "    [{idx}] material={} vertices={} triangles={} indices={} reps={}",
            prim.material_index,
            prim.vertices.len(),
            prim.triangle_count,
            prim.indices.len(),
            prim.repeating_mesh_reps,
        );
        println!(
            "        vertex_size={} init_flags={:#06b} pos_scale={:?} pos_bias={:?}",
            prim.vertex_size, prim.init_flags, prim.pos_scale, prim.pos_bias,
        );
        // Decoded position range — should sit inside the bounding box if the decode is correct.
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for v in &prim.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(v.pos[axis]);
                max[axis] = max[axis].max(v.pos[axis]);
            }
        }
        println!("        decoded pos  min {min:?} max {max:?}");
        if let Some(v) = prim.vertices.first() {
            println!("        vertex[0]    pos={:?} normal={:?} uv={:?}", v.pos, v.normal, v.uv);
        }
    }

    Ok(())
}
