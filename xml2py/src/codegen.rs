use std::collections::BTreeMap;

use heck::AsSnakeCase;

use crate::groups::Interface;
use crate::lookups::INPUT_ALIASES;
use eyesight_xml::nodes::{INode, Node};
use eyesight_xml::schema::{Eyesight, Group, Link, Named};

pub fn the_big_kahuna(eyesight: &Eyesight) -> String {
    let interfaces = crate::groups::check_interfaces(eyesight);

    let mut file = String::new();

    for group in &eyesight.groups {
        let Some(interface) = interfaces.get(&group.name) else {
            continue;
        };

        let function_body = group_to_python(group, interface);

        file += &format!("def node_group_{}():\n", AsSnakeCase(&group.name));

        for line in function_body {
            file += "    ";
            file += &line;
            file += "\n";
        }
        file += "\n\n";
    }

    file
}

fn group_to_python(group: &Group, interface: &Interface) -> Vec<String> {
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

    for (name, data_type) in &interface.inputs {
        lines.push(format!(
            "graph.input(bpy.types.{}, \"{name}\")",
            data_type.python_type()
        ));
    }
    for (name, data_type) in &interface.outputs {
        lines.push(format!(
            "graph.output(bpy.types.{}, \"{name}\")",
            data_type.python_type()
        ));
    }

    lines.push("".into());

    let tiers = topographic_sort(&group.shader.nodes, &group.shader.links);

    let mut inbound_edges = BTreeMap::<&str, Vec<&Link>>::new();
    for link in &group.shader.links {
        inbound_edges.entry(&link.to_node).or_default().push(link);
    }

    let x_coordinates = (0..).step_by(100);
    let y_coordinates = (0..).step_by(200);

    let node_iterator = tiers.iter().zip(x_coordinates).flat_map(|(tier, x)| {
        tier.iter()
            .zip(std::iter::repeat(x))
            .zip(y_coordinates.clone())
    });

    for ((node, x), y) in node_iterator {
        let var_name = node.name();
        let type_name = node.python_type();

        lines.extend([
            format!("{var_name} = graph.node("),
            format!("    bpy.types.{type_name},"),
            format!("    location=({x}, {y}),"),
        ]);

        let mut inputs = Vec::<(String, String)>::new();

        for link in inbound_edges
            .get(&var_name)
            .map(|x| &**x)
            .unwrap_or_default()
        {
            let src_node = &link.from_node;
            let src_socket = &link.from_socket;
            inputs.push((
                link.to_socket.clone(),
                format!("{src_node}[\"{src_socket}\"]"),
            ))
        }

        for input in node.inputs() {
            inputs.push((input.name.clone(), input.value.to_string()));
        }

        if !inputs.is_empty() {
            lines.push("    inputs={".into());

            for (dst_socket, value) in inputs {
                let mut dst_socket = &*dst_socket;
                if let Some(alias) = INPUT_ALIASES.get(type_name).and_then(|m| m.get(dst_socket)) {
                    dst_socket = alias;
                }

                lines.push(format!("        \"{dst_socket}\": {value},"));
            }

            lines.push("    },".into());
        }

        lines.push(")".into());
        lines.push("".into());
    }

    lines.push("return graph.tree".into());

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
