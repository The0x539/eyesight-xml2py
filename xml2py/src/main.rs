#[macro_use]
pub mod nodes;

mod schema;

pub mod distill;
pub mod groups;

mod codegen;

mod lookups;

use std::collections::BTreeMap;

use heck::{ToPascalCase, ToSnakeCase, ToTrainCase};
use nodes::Node;
use schema::{Eyesight, Named};

const SETTINGS_XML: &str =
    include_str!("/mnt/c/program files/studio 2.0/photorealisticrenderer/win/64/settings.xml");
const CUSTOM_XML: &str =
    include_str!("/mnt/c/program files/studio 2.0/data/CustomColors/CustomColorSettings.xml");

fn main() {
    let eyesight_main = quick_xml::de::from_str::<schema::Eyesight>(SETTINGS_XML).unwrap();
    let eyesight_custom = quick_xml::de::from_str::<schema::Eyesight>(CUSTOM_XML).unwrap();
    let mut eyesight = merge_eyesight(eyesight_main, eyesight_custom);

    beautify_names(&mut eyesight);

    let s = codegen::the_big_kahuna(&eyesight);
    println!("{s}");

    // groups::check_interfaces(&eyesight);
    // distill::distill_materials(&eyesight.materials);
}

fn beautify_names(eyesight: &mut Eyesight) {
    for group in &mut eyesight.groups {
        let pascal_name = group.name.to_pascal_case();

        for node in &mut group.shader.nodes {
            beautify_node_name(node.name_mut(), &pascal_name);
            if let Node::Group(g) = node {
                beautify_group_name(&mut g.group_name);
            }
        }

        for link in &mut group.shader.links {
            beautify_node_name(&mut link.from_node, &pascal_name);
            beautify_node_name(&mut link.to_node, &pascal_name);
        }

        beautify_group_name(&mut group.name);
    }

    for material in &mut eyesight.materials {
        for node in &mut material.shader.nodes {
            if let Node::Group(g) = node {
                beautify_group_name(&mut g.group_name);
            }
        }
    }
}

fn beautify_node_name(name: &mut String, group_name: &str) {
    *name = name
        .replace("Anique", "Antique")
        .trim_start_matches(group_name)
        .to_snake_case();
}

fn beautify_group_name(name: &mut String) {
    *name = name
        .trim_end_matches("-GROUP")
        .to_train_case()
        .replace("-", " ");
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
