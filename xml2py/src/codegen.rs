use std::collections::BTreeMap;

use crate::groups::Interface;
use crate::lookups::INPUT_ALIASES;
use crate::nodes::{INode, Node};
use crate::schema::{Eyesight, Group, Link, Named};

pub fn the_big_kahuna(eyesight: &Eyesight) -> String {
    let interfaces = crate::groups::check_interfaces(eyesight);

    let solid_group = eyesight.groups.iter().find(|g| g.name == "Solid").unwrap();

    let solid_group_interface = &interfaces["Solid"];

    let lines = groupp(solid_group, solid_group_interface);
    lines.join("\n")
}

fn groupp(group: &Group, _interface: &Interface) -> Vec<String> {
    let mut lines = vec![];

    let group_name = &group.name;
    lines.extend([
        format!("if tree := bpy.data.node_groups.get(\"{group_name}\"):"),
        format!("    return tree"),
        format!(""),
        format!("tree = bpy.data.node_groups.new(\"{group_name}\", \"ShaderNodeTree\")"),
        format!("graph = NodeGraph(tree)"),
        format!(""),
    ]);

    let tiers = topographic_sort(&group.shader.nodes, &group.shader.links);

    let mut inbound_edges = BTreeMap::<&str, Vec<&Link>>::new();
    for link in &group.shader.links {
        inbound_edges.entry(&link.to_node).or_default().push(link);
    }

    let mut x = 0;
    for tier in tiers {
        let mut y = 0;
        for node in tier {
            let var_name = node.name();
            let type_name = node.python_type();

            lines.extend([
                format!("{var_name} = graph.node("),
                format!("    bpy.types.{type_name},"),
                format!("    location=({x}, {y}),"),
            ]);

            if let Some(links) = inbound_edges.get(&var_name) {
                lines.push("    inputs={".into());
                for link in links {
                    let mut dst_socket = &*link.to_socket;
                    if let Some(alias) =
                        INPUT_ALIASES.get(type_name).and_then(|m| m.get(dst_socket))
                    {
                        dst_socket = alias;
                    }

                    let src_node = &link.from_node;
                    let src_socket = &link.from_socket;
                    lines.push(format!(
                        "        \"{dst_socket}\": {src_node}[\"{src_socket}\"],"
                    ));
                }
                lines.push("    },".into());
            }

            lines.extend([format!(")"), format!("")]);

            y += 200;
        }
        x += 100;
    }

    lines
}

fn topographic_sort<'a>(nodes: &'a [Node], links: &[Link]) -> Vec<Vec<&'a Node>> {
    let mut outbound_edges = BTreeMap::<&str, Vec<&str>>::new();
    for link in links {
        outbound_edges
            .entry(&link.from_node)
            .or_default()
            .push(&link.to_node);
    }

    let mut unvisited = nodes
        .iter()
        .map(|n| (n.name(), n))
        .collect::<BTreeMap<_, _>>();

    let mut tiers = vec![];
    let mut current_tier = vec![];

    // gather the nodes with an in-degree of 0
    current_tier.extend(nodes.iter().filter(|n| {
        !outbound_edges
            .values()
            .flatten()
            .any(|dst_node_name| n.name() == *dst_node_name)
    }));
    for n in &current_tier {
        unvisited.remove(n.name()).unwrap();
    }

    // breadth-first search
    while !current_tier.is_empty() {
        let mut next_tier = vec![];
        for src_node in &current_tier {
            let Some(out_links) = outbound_edges.get(src_node.name()) else {
                continue;
            };
            for out_link in out_links {
                if let Some(dst_node) = unvisited.remove(out_link) {
                    next_tier.push(dst_node);
                }
            }
        }

        tiers.push(current_tier);
        current_tier = next_tier;
    }

    assert!(unvisited.is_empty());

    tiers
}
