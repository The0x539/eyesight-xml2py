use std::collections::{HashMap, HashSet};

use crate::{
    nodes::{GroupReference, Node, SocketType},
    schema::{Eyesight, Group},
};

pub fn check_interfaces(eyesight: &Eyesight) -> HashMap<String, Interface> {
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

    let mut complete = HashMap::new();

    for (name, incomplete) in interfaces {
        if unused_groups.contains(&name) {
            continue;
        }

        let mut interface = Interface::default();
        for (socket_name, data_type) in incomplete.inputs {
            let data_type = data_type.unwrap_or_else(|| panic!("{name} / {socket_name}"));
            interface.inputs.insert(socket_name, data_type);
        }
        for (socket_name, data_type) in incomplete.outputs {
            let data_type = data_type.unwrap_or_else(|| panic!("{name} / {socket_name}"));
            interface.outputs.insert(socket_name, data_type);
        }
        complete.insert(name, interface);
    }

    complete
}

fn discover_sockets(group: &Group) -> IncompleteInterface {
    let mut interface = IncompleteInterface::default();

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

fn check_socket_types(
    node: &GroupReference,
    interfaces: &mut HashMap<String, IncompleteInterface>,
) {
    let interface = interfaces.get_mut(&node.group_name).unwrap();

    for input in &node.inputs {
        check_socket_type(&mut interface.inputs, &input.name, input.data_type);
    }

    for output in &node.outputs {
        check_socket_type(&mut interface.outputs, &output.name, output.data_type);
    }
}

fn check_socket_type(
    sockets: &mut HashMap<String, Option<SocketType>>,
    name: &str,
    usage_type: SocketType,
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
struct IncompleteInterface {
    inputs: HashMap<String, Option<SocketType>>,
    outputs: HashMap<String, Option<SocketType>>,
}

#[derive(Default, Debug)]
pub struct Interface {
    pub inputs: HashMap<String, SocketType>,
    pub outputs: HashMap<String, SocketType>,
}
