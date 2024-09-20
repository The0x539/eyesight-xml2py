use schema::*;

pub mod schema;

fn main() {
    let settings = include_str!(
        "/mnt/c/program files/studio 2.0 earlyaccess/photorealisticrenderer/win/64/settings_camera_light.xml"
    );

    let eyesight: schema::Eyesight = quick_xml::de::from_str(settings).unwrap();

    let mut preset = eyesight
        .presets
        .into_iter()
        .find(|p| p.name == "MECHANIC_FL")
        .unwrap();

    println!("import bpy");
    println!("import mathutils");

    println!("for light in bpy.data.lights: bpy.data.lights.remove(light, do_unlink=True)");

    let matrix = [
        -0.173651278,
        0.899666607,
        -0.400556326,
        -2295.80151,
        5.96046448E-08,
        0.406735897,
        0.913545907,
        5581.06934,
        0.984807253,
        0.158638418,
        -0.070630312,
        -287.269836,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    preset.lights.push(Light {
        matrix,
        inner: LightInner {
            name: "Hardcoded Light 1".into(),
            inner: LightInnerInner {
                background_shader: None,
                emission: Some(LightNode {
                    name: String::new(),
                    inputs: vec![NodeInput::Float {
                        name: "Strength".into(),
                        value: 2.0,
                    }],
                }),
                ..Default::default()
            },
            light_type: LightType::Distant,
            size: 0.8,
            ..Default::default()
        },
    });

    for light in &preset.lights {
        let name = &light.inner.name;
        let light_type = match light.inner.light_type {
            LightType::Distant => "SUN",
            _ => todo!(),
        };
        let angle = 2.0 * light.inner.size.atan();
        println!("light = bpy.data.lights.new('{name}', '{light_type}')");
        println!("light.angle = {angle}");

        let mut energy = 0.0;
        let mut color = [1.0; 3];
        for input in &light.inner.inner.emission.as_ref().unwrap().inputs {
            match *input {
                NodeInput::Color { value, .. } => color = value,
                NodeInput::Float { value, .. } => energy = value,
            }
        }

        println!("light.energy = {energy}");
        println!("light.color = {color:?}");

        println!("light_obj = bpy.data.objects.new('{name}', light)");
        println!("light_obj.matrix_world = mathutils.Matrix((");
        for chunk in light.matrix.chunks(4) {
            println!("    {chunk:?},");
        }
        println!("))");
        println!(
            "bpy.context.view_layer.active_layer_collection.collection.objects.link(light_obj)"
        )
    }
}
