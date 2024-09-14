use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;

use heck::AsSnakeCase;

use crate::groups::Interface;
use crate::lookups::{INPUT_ALIASES, OUTPUT_ALIASES};
use eyesight_xml::nodes::{INode, Node};
use eyesight_xml::schema::{Eyesight, Group, Link};
use eyesight_xml::Named;

pub fn the_big_kahuna(eyesight: &Eyesight, groups_to_convert: &HashSet<&str>) -> String {
    let interfaces = crate::groups::check_interfaces(eyesight);

    let mut file = String::new();

    writeln!(file, "import bpy").unwrap();
    writeln!(file, "from .node_dsl import NodeGraph").unwrap();
    writeln!(
        file,
        "from .custom_nodes import node_group_uv_degradation, node_group_project_to_axis_plane"
    )
    .unwrap();
    writeln!(file).unwrap();

    for group in &eyesight.groups {
        if !groups_to_convert.contains(&*group.name) {
            continue;
        }

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

fn get_socket_key(s: &str) -> String {
    if let Ok(n) = s.parse::<u32>() {
        n.to_string()
    } else {
        format!("{s:?}") // escapes and quotes
    }
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

        for (name, val) in node.attributes() {
            lines.push(format!("    {name}={val},"));
        }

        let mut inputs = Vec::<(String, String)>::new();

        for link in inbound_edges
            .get(&var_name)
            .map(|x| &**x)
            .unwrap_or_default()
        {
            let src_node = &link.from_node;
            let mut src_socket = &*link.from_socket;

            if let Some(src_node_obj) = group.shader.nodes.iter().find(|n| n.name() == src_node) {
                if let Some(alias_table) = OUTPUT_ALIASES.get(src_node_obj.python_type()) {
                    if let Some(alias) = alias_table.get(src_socket) {
                        src_socket = alias;
                    }
                }
            }

            let src_socket = get_socket_key(src_socket);
            inputs.push((link.to_socket.clone(), format!("{src_node}[{src_socket}]")))
        }

        for input in node.inputs_override() {
            inputs.push((input.name.clone(), input.value.to_string()));
        }

        if !inputs.is_empty() {
            lines.push("    inputs={".into());

            for (dst_socket, value) in inputs {
                let mut dst_socket = &*dst_socket;
                if let Some(alias) = INPUT_ALIASES.get(type_name).and_then(|m| m.get(dst_socket)) {
                    dst_socket = alias;
                }
                let dst_socket = get_socket_key(dst_socket);

                lines.push(format!("        {dst_socket}: {value},"));
            }

            lines.push("    },".into());
        }

        lines.push(")".into());
        lines.extend(node.after());
        lines.push("".into());
    }

    lines.push("return graph.tree".into());

    lines
}

fn topographic_sort<'a>(nodes: &'a [Node], links: &[Link]) -> Vec<Vec<&'a Node>> {
    let mut inbound_edges = BTreeMap::<&str, BTreeSet<&str>>::new();
    for link in links {
        inbound_edges
            .entry(&link.to_node)
            .or_default()
            .insert(&link.from_node);
    }

    let mut visited = BTreeSet::new();

    let mut tiers = vec![];

    loop {
        let current_tier = nodes
            .iter()
            .filter(|n| !visited.contains(n.name()))
            .filter(|n| inbound_edges.get(n.name()).is_none_or(|v| v.is_empty()))
            .collect::<Vec<_>>();

        if current_tier.is_empty() {
            break;
        }

        for node in &current_tier {
            for edges in inbound_edges.values_mut() {
                edges.remove(node.name());
            }
            visited.insert(node.name());
        }

        tiers.push(current_tier);
    }

    tiers
}
