use serde_derive::Deserialize;
use xml2py_macros::node;

pub trait Named {
    fn name(&self) -> &str;
    fn name_mut(&mut self) -> &mut String;
}

#[derive(Deserialize, Debug)]
pub struct Eyesight {
    #[serde(default, rename = "material")]
    pub materials: Vec<Material>,
    #[serde(default, rename = "group")]
    pub groups: Vec<Group>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(rename = "@name")]
    pub name: String,
    pub shader: Shader,
}

impl Named for Group {
    fn name(&self) -> &str {
        &self.name
    }
    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

enums! {
    DisplacementMethod { Bump }
    VolumeInterpolationMethod { Linear }
    VolumeSamplingMethod { MultipleImportance }
}

#[node]
pub struct Material {
    name: String,
    displacement_method: DisplacementMethod,
    heterogeneous_volume: bool,
    use_local_tuning: bool,
    use_mis: bool,
    use_transparent_shadow: bool,
    volume_interpolation_method: VolumeInterpolationMethod,
    volume_sampling_method: VolumeSamplingMethod,
    diffuse_ao_factor: Option<f32>,
    glossy_ao_factor: Option<f32>,
    subsurface_ao_factor: Option<f32>,
    subsurface_factor: Option<f32>,
    transmission_ao_factor: Option<f32>,

    #[rename = "shader"]
    shader: Shader,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    #[serde(rename = "$value")]
    pub nodes: Vec<crate::nodes::Node>,
    #[serde(rename = "connect")]
    pub links: Vec<Link>,
}

#[derive(Deserialize, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(rename = "@from_node")]
    pub from_node: String,
    #[serde(rename = "@to_node")]
    pub to_node: String,
    #[serde(rename = "@from_socket")]
    pub from_socket: String,
    #[serde(rename = "@to_socket")]
    pub to_socket: String,
}

impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Link({}.{} -> {}.{})",
            self.from_node, self.from_socket, self.to_node, self.to_socket
        )
    }
}
