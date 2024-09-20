use sqlx::SqliteConnection;

use eyesight_xml::{
    nodes::{Node, Vec3},
    schema::{Eyesight, Material},
    Named,
};

pub async fn insert_file(
    contents: &str,
    name: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let solid_template: Material = quick_xml::de::from_str(SOLID_TEMPLATE)?;
    let trans_template: Material = quick_xml::de::from_str(TRANS_TEMPLATE)?;

    let db_id = sqlx::query_scalar!(
        "INSERT INTO eyesight_database VALUES (NULL, ?) RETURNING id",
        name
    )
    .fetch_one(&mut *conn)
    .await?;

    let eyesight: Eyesight = quick_xml::de::from_str(contents)?;

    for mut material in eyesight.materials {
        let (material_name, rgb) = excise(&mut material);
        let category = if material == solid_template {
            "solid"
        } else if material == trans_template {
            "transparent"
        } else {
            continue;
        };

        sqlx::query!(
            "INSERT INTO eyesight_color VALUES (?, ?, ?, ?, ?, ?)",
            db_id,
            material_name,
            category,
            rgb.0[0],
            rgb.0[1],
            rgb.0[2],
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

fn excise(material: &mut Material) -> (String, Vec3) {
    let name = std::mem::take(&mut material.name);

    let mut color = Vec3::default();

    for node in &mut material.shader.nodes {
        if let Node::Group(group) = node {
            group.inputs_.sort_by_key(|i| i.name.clone());
            group.outputs.sort_by_key(|o| o.name.clone());
        } else if let Node::Color(color_node) = node {
            if color_node.name == "RGB" {
                color = std::mem::take(&mut color_node.value);
            }
        }
    }

    material.shader.nodes.sort_by_key(|n| n.name().to_owned());
    material.shader.links.sort();

    (name, color)
}

// TODO: move to one or more external XML files

const SOLID_TEMPLATE: &str = r#"
    <material displacement_method="bump" heterogeneous_volume="False" name="" use_local_tuning="False" use_mis="True" use_transparent_shadow="True" volume_interpolation_method="linear" volume_sampling_method="multiple_importance">
        <shader>
            <color name="RGB" value="0 0 0" />
            <group group_name="SOLID-GROUP" name="SOLID-GROUP">
                <input name="BaseColor" type="color" />
                <input name="SubsurfaceColor" type="color" />
                <output name="Shader" type="closure" />
            </group>
            <connect from_node="RGB" from_socket="Color" to_node="SOLID-GROUP" to_socket="BaseColor" />
            <connect from_node="RGB" from_socket="Color" to_node="SOLID-GROUP" to_socket="SubsurfaceColor" />
            <connect from_node="SOLID-GROUP" from_socket="Shader" to_node="Output" to_socket="Surface" />
        </shader>
    </material>
"#;

const TRANS_TEMPLATE: &str = r#"
    <material displacement_method="bump" heterogeneous_volume="False" name="" use_local_tuning="False" use_mis="True" use_transparent_shadow="True" volume_interpolation_method="linear" volume_sampling_method="multiple_importance">
        <shader>
            <color name="RGB" value="0 0 0" />
            <color name="RGB_White" value="1.0 1.0 1.0" />
            <group group_name="TRANS-GROUP_BASE" name="TRANS-GROUP_BASE">
                <input name="Color" type="color" />
                <input name="WhiteColor" type="color" />
                <output name="Normal" type="vector" />
                <output name="Shader" type="closure" />
                <output name="Volume" type="closure" />
            </group>
            <connect from_node="RGB" from_socket="Color" to_node="TRANS-GROUP_BASE" to_socket="Color" />
            <connect from_node="RGB_White" from_socket="Color" to_node="TRANS-GROUP_BASE" to_socket="WhiteColor" />
            <connect from_node="TRANS-GROUP_BASE" from_socket="Shader" to_node="Output" to_socket="Surface" />
            <connect from_node="TRANS-GROUP_BASE" from_socket="Volume" to_node="Output" to_socket="Volume" />
        </shader>
    </material>
"#;
