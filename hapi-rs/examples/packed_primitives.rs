#![allow(dead_code)]
#![allow(unused)]

use hapi_rs::geometry::{PackedPrimInstancingMode as IM, *};
use hapi_rs::node::*;
use hapi_rs::session::*;
use hapi_rs::{PartType, Result};

fn main() -> Result<()> {
    let mut session = new_in_process()?;
    session.initialize(&SessionOptions::default())?;

    let lib = session.load_asset_file("otls/sesi/PackedPrimitive.hda")?;
    let asset = lib.try_create_first()?;
    let mut co = CookOptions::default();
    for mode in [IM::Disabled, IM::Hierarchy, IM::Flat] {
        println!("Using PackedPrimInstancingMode::{}", match mode {
            PackedPrimInstancingMode::Disabled => "Disabled",
            PackedPrimInstancingMode::Hierarchy => "Hierarchy",
            PackedPrimInstancingMode::Flat => "Flat",
            _ => unreachable!()
        });
        co.set_packed_prim_instancing_mode(mode);
        asset.cook_blocking(Some(&co))?;

        let nodes = asset.get_children(NodeType::Sop, NodeFlags::Any, false)?;
        for handle in nodes {
            let node = handle.to_node(&session)?;
            node.cook_blocking(Some(&co))?;
            let geo = node.geometry()?.expect("geometry");
            println!("Part count for geo {}: {}", geo.node.handle.0, geo.info.part_count());
            for part in geo.partitions()? {
                println!(
                    "Part {}\n   Point Count = {}\n{}",
                    part.part_id(),
                    part.point_count(),
                    match part.part_type() {
                        PartType::Mesh => {
                            format!("   Type = Mesh")
                        }
                        PartType::Curve => {
                            format!("   Type = Curve")
                        }
                        PartType::Instancer => {
                            format!("   Type = Instancer")
                        }
                        p => "oops".to_string(),
                    }
                );
            }
        }
    }
    Ok(())
}
