use std::collections::BTreeMap;

use eyesight_xml::nodes::{Node, NodeInputValue};
use eyesight_xml::schema::Material;
use eyesight_xml::Named;

pub fn distill_materials(materials: &[Material]) {
    let mut materials = materials
        .iter()
        .cloned()
        .map(|m| (m.name.clone(), m))
        .collect::<BTreeMap<_, _>>();

    let mut f = |name: &str| {
        let zero = without_color(materials[name].clone());
        extract_colors_like(&zero, &mut materials)
    };

    let _solid = f("SOLID-BLUE");
    let _chrome = f("CHROME-GREEN");
    let _glitter = f("GLITTER-TRANS_BRIGHT_GREEN");
    let _metal = f("METAL-COPPER");
    let _milky = f("MILKY-GLOW_IN_DARK_RED");
    // milky opaque/trans/white / non-gitd seem to each be slightly unique
    let _pearl_flat = f("PEARL-BLUE");
    let _pearl = f("PEARL-GOLD");
    let _rubber = f("RUBBER-BLUE");
    let _rubber_trans = f("RUBBER-TRANS_RED");
    let _satin = f("SATIN-TRANS_DARK_PINK");
    let _speckle = f("SPECKLE-BLACK_GOLD");
    let _trans = f("TRANS-AQUA");
    let _glowing_neon = f("TRANS-GLOWING_NEON_MAGENTA");
    let _luminous = f("TRANS-LUMINOUS_CYAN");
    let _luminous_soft = f("TRANS-LUMINOUS_SOFT_GREEN");
    let _trans_neon = f("TRANS-NEON_YELLOW");
    let _translucent = f("TRANS-TRANSLUCENT_LIGHT_BLUE");

    for name in materials.keys() {
        println!("{name}");
    }
}

fn extract_colors_like(
    zero: &Material,
    materials: &mut BTreeMap<String, Material>,
) -> Vec<Material> {
    let names = materials.keys().cloned().collect::<Vec<_>>();

    let mut matches = vec![];
    for name in &names {
        let uncolored = without_color(materials[name].clone());
        if uncolored == *zero {
            matches.push(materials.remove(name).unwrap());
        }
    }
    matches
}

fn without_color(mut material: Material) -> Material {
    for node in &mut material.shader.nodes {
        if let Node::Color(color_node) = node {
            if matches!(
                &*color_node.name,
                "RGB" | "RGB_GlowDark" | "RGB_Chip" | "RGB_White" | "RGB_Second"
            ) {
                color_node.value.0 = [0.0; 3];
            }
        } else if let Node::Group(group_node) = node {
            if matches!(
                &*group_node.group_name,
                "PEARL-GROUP" | "PEARL-FLAT-GROUP" | "SATIN-GROUP"
            ) {
                for input in &mut group_node.inputs_ {
                    input.value = Some(NodeInputValue::Float(0.0));
                }
            }
        } else if let Node::Value(value_node) = node {
            if value_node.name == "XOffset" {
                value_node.value = 0.0;
            }
        } else if let Node::Emission(emission_node) = node {
            for input in &mut emission_node.inputs {
                if input.name == "Color" {
                    input.value = NodeInputValue::Color(Default::default())
                }
            }
        }
    }

    material.name.clear();
    material.shader.nodes.sort_by_key(|n| n.name().to_owned());
    material.shader.links.sort();

    material
}
