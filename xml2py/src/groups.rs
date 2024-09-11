use std::collections::{HashMap, HashSet};

use crate::{
    nodes::{GroupReference, Node, NodeInputType},
    schema::{Eyesight, Group},
};

pub fn check_interfaces(eyesight: &Eyesight) {
    let mut interfaces = eyesight
        .groups
        .iter()
        .map(|group| (group.name.clone(), discover_sockets(&group)))
        .collect::<HashMap<_, _>>();

    let mut unused_groups = interfaces.keys().cloned().collect::<HashSet<_>>();

    for material in &eyesight.materials {
        for node in &material.shader.nodes {
            if let Node::Group(node) = node {
                check_socket_types(node, &mut interfaces);
                unused_groups.remove(&node.group_name);
            }
        }
    }

    for (name, interface) in &interfaces {
        if unused_groups.contains(name) {
            continue;
        }
        for input in &interface.inputs {
            assert!(input.1.is_some());
        }
        for input in &interface.outputs {
            assert!(input.1.is_some());
        }
    }
}

fn discover_sockets(group: &Group) -> Interface {
    let mut interface = Interface::default();

    let mut input_node_name = None;
    let mut output_node_name = None;
    for node in &group.shader.nodes {
        if let Node::GroupInput(n) = node {
            input_node_name = Some(&n.name);
        } else if let Node::GroupOutput(n) = node {
            output_node_name = Some(&n.name);
        }
    }

    for link in &group.shader.links {
        if Some(&link.from_node) == input_node_name {
            interface.inputs.insert(link.from_socket.clone(), None);
        } else if Some(&link.to_node) == output_node_name {
            interface.outputs.insert(link.to_socket.clone(), None);
        }
    }

    interface
}

fn check_socket_types(node: &GroupReference, interfaces: &mut HashMap<String, Interface>) {
    let interface = interfaces.get_mut(&node.group_name).unwrap();

    for input in &node.inputs {
        check_socket_type(&mut interface.inputs, &input.name, input.data_type);
    }

    for output in &node.outputs {
        check_socket_type(&mut interface.outputs, &output.name, output.data_type);
    }
}

fn check_socket_type(
    sockets: &mut HashMap<String, Option<NodeInputType>>,
    name: &str,
    usage_type: NodeInputType,
) {
    let data_type = sockets.entry(name.into()).or_insert_with(|| {
        println!("unknown socket: {name} {usage_type:?}");
        None
    });

    match data_type {
        None => *data_type = Some(usage_type),
        Some(existing) => assert_eq!(usage_type, *existing),
    }
}

#[derive(Default, Debug)]
struct Interface {
    inputs: HashMap<String, Option<NodeInputType>>,
    outputs: HashMap<String, Option<NodeInputType>>,
}
