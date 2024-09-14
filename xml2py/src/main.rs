pub mod distill;
pub mod groups;

mod codegen;

mod lookups;

use std::collections::{BTreeMap, HashSet};

use eyesight_xml::nodes::{Node, NodeInput, NodeInputValue, VectorMath, VectorOperation};
use eyesight_xml::schema::{Eyesight, Link, Shader};
use eyesight_xml::Named;
use heck::{ToPascalCase, ToSnakeCase, ToTitleCase};

const SETTINGS_XML: &str =
    include_str!("/mnt/c/program files/studio 2.0/photorealisticrenderer/win/64/settings.xml");
const CUSTOM_XML: &str =
    include_str!("/mnt/c/program files/studio 2.0/data/CustomColors/CustomColorSettings.xml");

fn main() {
    let eyesight_main = quick_xml::de::from_str::<Eyesight>(SETTINGS_XML).unwrap();
    let eyesight_custom = quick_xml::de::from_str::<Eyesight>(CUSTOM_XML).unwrap();
    let mut eyesight = merge_eyesight(eyesight_main, eyesight_custom);

    beautify_names(&mut eyesight);

    // handle vector average nodes, stupid annoying ugh
    for shader in eyesight.all_shaders_mut() {
        implement_vector_average(shader);
    }

    let mut visited = HashSet::<&str>::new();
    let mut unvisited = vec!["Solid", "Trans Group Base"];

    while let Some(name) = unvisited.pop() {
        let group = eyesight.groups.iter().find(|g| g.name == name).unwrap();

        for node in &group.shader.nodes {
            if let Node::Group(gr) = node {
                if !visited.contains(&*gr.group_name) {
                    unvisited.push(&gr.group_name);
                }
            }
        }

        visited.insert(name);
    }

    // println!("{visited:?}");

    let s = codegen::the_big_kahuna(&eyesight, &visited);
    println!("{s}");

    // groups::check_interfaces(&eyesight);
    // distill::distill_materials(&eyesight.materials);
}

// <vector_math type="average"> doesn't actually have a direct equivalent in Blender,
// so emulate it using <vector_math type="add"> -> <vector_math type="scale">
fn implement_vector_average(shader: &mut Shader) {
    // index-based iteration lets us push to shader.nodes inside the loop body
    for i in 0..shader.nodes.len() {
        let node = &mut shader.nodes[i];
        let Node::VectorMath(vector_math) = node else {
            continue;
        };

        if vector_math.operation != VectorOperation::Average {
            continue;
        };

        // first, add the two inputs
        vector_math.operation = VectorOperation::Add;
        let add = vector_math;

        // then, halve the result
        let scale = VectorMath {
            operation: VectorOperation::Scale,
            name: add.name.clone() + "_part_2",
            inputs: vec![NodeInput {
                name: "Scale".into(),
                value: NodeInputValue::Float(0.5),
            }],
        };

        // take anything that was reading from node 1 and reconnect it to node 2
        for link in &mut shader.links {
            if link.from_node == add.name {
                link.from_node = scale.name.clone();
            }
        }

        let link = Link::new(&add.name, "Vector", &scale.name, "0");
        shader.nodes.push(Node::VectorMath(scale));
        shader.links.push(link);
    }
}

fn beautify_names(eyesight: &mut Eyesight) {
    for group in &mut eyesight.groups {
        let pascal_name = group.name.to_pascal_case();

        for node in &mut group.shader.nodes {
            beautify_node_name(node.name_mut(), &group.name, &pascal_name);
            if let Node::Group(g) = node {
                beautify_group_name(&mut g.group_name);
            }
        }

        for link in &mut group.shader.links {
            beautify_node_name(&mut link.from_node, &group.name, &pascal_name);
            beautify_node_name(&mut link.to_node, &group.name, &pascal_name);
        }

        beautify_group_name(&mut group.name);
    }

    for material in &mut eyesight.materials {
        for node in &mut material.shader.nodes {
            if let Node::Group(g) = node {
                beautify_group_name(&mut g.group_name);
            }
        }

        beautify_material_name(&mut material.name)
    }
}

fn beautify_node_name(name: &mut String, original_group_name: &str, pascal_group_name: &str) {
    *name = name
        .replace("Anitique", "Antique")
        .replace("Anique", "Antique")
        .replace("Ghrome", "Chrome")
        .trim_start_matches(original_group_name)
        .trim_start_matches(pascal_group_name)
        .to_snake_case();
}

fn beautify_group_name(name: &mut String) {
    *name = name
        .trim_end_matches("-GROUP")
        .trim_end_matches("Group")
        .to_title_case()
}

fn beautify_material_name(name: &mut String) {
    *name = name
        .to_title_case()
        .replace("Trans Trans", "Trans")
        .replace("Trans ", "Trans-")
}

fn merge<T: Named + PartialEq + std::fmt::Debug>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    let mut map = a
        .into_iter()
        .map(|x| (x.name().to_owned(), x))
        .collect::<BTreeMap<_, _>>();

    for x in b {
        if let Some(conflict) = map.get(x.name()) {
            assert_eq!(*conflict, x);
        } else {
            map.insert(x.name().to_owned(), x);
        }
    }

    map.into_values().collect()
}

fn merge_eyesight(a: Eyesight, b: Eyesight) -> Eyesight {
    Eyesight {
        materials: merge(a.materials, b.materials),
        groups: merge(a.groups, b.groups),
    }
}
