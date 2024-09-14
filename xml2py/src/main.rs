pub mod distill;
pub mod groups;

mod codegen;

mod lookups;

use std::collections::{BTreeMap, HashSet};

use eyesight_xml::nodes::Node;
use eyesight_xml::schema::Eyesight;
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

    let mut visited = HashSet::<&str>::new();
    let mut unvisited = vec!["Solid"];

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

    let s = codegen::the_big_kahuna(&eyesight, &visited);
    println!("{s}");

    // groups::check_interfaces(&eyesight);
    // distill::distill_materials(&eyesight.materials);
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
