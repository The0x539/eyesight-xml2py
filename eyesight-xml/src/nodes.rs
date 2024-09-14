u,se std::borrow::Cow;
use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use serde::de::{Deserialize, Deserializer, Error, Unexpected};
use serde_derive::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use xml2py_macros::node;

use crate::Named;

pub trait INode: Named {
    const PYTHON_TYPE: &str;
    fn python_type(&self) -> &'static str {
        Self::PYTHON_TYPE
    }
    fn inputs(&self) -> &[NodeInput] {
        &[]
    }
    fn inputs_override(&self) -> Vec<NodeInput> {
        self.inputs().to_vec()
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![]
    }
    fn after(&self) -> Vec<String> {
        vec![]
    }
}

fn python_enum(x: impl Debug) -> String {
    // Hideous.
    format!("'{}'", format!("{x:?}").to_shouty_snake_case())
}

#[node]
struct NodeInput {
    name: String,
    #[serde(flatten)]
    value: NodeInputValue,
}

#[serde_as]
#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum NodeInputValue {
    Float(#[serde_as(as = "DisplayFromStr")] f32),
    Vector(Vec3),
    Int(#[serde_as(as = "DisplayFromStr")] u32),
    Color(Vec3),
    Boolean(#[serde(deserialize_with = "py_bool")] bool),
}

impl std::fmt::Display for NodeInputValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(n) => write!(f, "{n}"),
            Self::Vector(v) => write!(f, "{v}"),
            Self::Color(Vec3([r, g, b])) => write!(f, "({r}, {g}, {b}, 1.0)"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Boolean(b) => f.write_str(if *b { "True" } else { "False" }),
        }
    }
}

impl SocketType {
    pub fn python_type(&self) -> &'static str {
        match self {
            Self::Float => "NodeSocketFloat",
            Self::Vector => "NodeSocketVector",
            Self::Int => "NodeSocketInt",
            Self::Color => "NodeSocketColor",
            Self::Boolean => "NodeSocketBoolean",
            Self::Closure => "NodeSocketShader",
        }
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for SocketType {
    type Value = NodeInputValue;

    fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        match self {
            Self::Float => Deserialize::deserialize(de).map(NodeInputValue::Float),
            Self::Vector => Deserialize::deserialize(de).map(NodeInputValue::Vector),
            Self::Int => Deserialize::deserialize(de).map(NodeInputValue::Int),
            Self::Color => Deserialize::deserialize(de).map(NodeInputValue::Color),
            Self::Boolean => Deserialize::deserialize(de).map(NodeInputValue::Boolean),
            Self::Closure => Err(D::Error::custom("somehow found a default closure value")),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GroupReferenceInput {
    pub name: String,
    pub data_type: SocketType,
    pub value: Option<NodeInputValue>,
}

impl<'de> Deserialize<'de> for GroupReferenceInput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GroupReferenceInput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("@name, @type, and possibly @value")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                enum Field {
                    #[serde(rename = "@name")]
                    Name,
                    #[serde(rename = "@type")]
                    Type,
                    #[serde(rename = "@value")]
                    Value,
                }

                let mut name = None::<String>;
                let mut data_type = None::<SocketType>;
                let mut value = None::<NodeInputValue>;

                while let Some(field) = map.next_key::<Field>()? {
                    match field {
                        Field::Name => {
                            if name.is_some() {
                                return Err(A::Error::duplicate_field("@name"));
                            } else {
                                name = Some(map.next_value()?);
                            }
                        }
                        Field::Type => {
                            if data_type.is_some() {
                                return Err(A::Error::duplicate_field("@type"));
                            } else {
                                data_type = Some(map.next_value()?);
                            }
                        }
                        Field::Value => {
                            if value.is_some() {
                                return Err(A::Error::duplicate_field("@value"));
                            } else if let Some(data_type) = data_type {
                                value = Some(map.next_value_seed(data_type)?);
                            } else {
                                return Err(A::Error::custom("encountered @value before @type"));
                            }
                        }
                    }
                }

                let name = name.ok_or_else(|| A::Error::missing_field("@name"))?;
                let data_type = data_type.ok_or_else(|| A::Error::missing_field("@type"))?;

                Ok(GroupReferenceInput {
                    name,
                    data_type,
                    value,
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vec3(pub [f32; 3]);

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&(self.0[0], self.0[1], self.0[2]), f)
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = float_seq(de)?;
        let arr = <[f32; 3]>::try_from(v).map_err(|v| D::Error::invalid_length(v.len(), &"3"))?;
        Ok(Self(arr))
    }
}

fn float_seq<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<f32>, D::Error> {
    let s: Cow<'de, str> = Deserialize::deserialize(de)?;
    let mut v = vec![];
    for x in s.split_whitespace() {
        let n = x
            .trim_end_matches(',')
            .parse::<f32>()
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(x), &"a float"))?;
        v.push(n)
    }
    Ok(v)
}

fn py_bool<'de, D: Deserializer<'de>, T: From<bool>>(de: D) -> Result<T, D::Error> {
    let s: Cow<'de, str> = Deserialize::deserialize(de)?;
    match &*s {
        "True" | "true" => Ok(true.into()),
        "False" | "false" => Ok(false.into()),
        _ => Err(D::Error::invalid_value(
            Unexpected::Str(&s),
            &"true or false",
        )),
    }
}

enums! {
    SocketType {
        Float,
        Vector,
        Int,
        Color,
        Boolean,
        Closure,
    }

    MathOperation {
        Add,
        Multiply,
        Subtract,
        Divide,
        Floor,
        Minimum,
        Maximum,
        LessThan,
        Power,
    }

    Axis { X, Y, Z }
    VectorOperation { Average, Multiply, Add, Scale }
    BsdfDistribution { Ggx }
    Projection { Flat }
    VectorType { Point }
    VectorSpace { Object, World }
    MixOperation { Darken, Mix }
    Interpolation { Linear }
    TexMappingType { Point, Texture }
    Extension { Repeat }
    ColorSpace { Color }
    MixType { Mix }
    SubsurfaceMethod { Burley }
    VoronoiColoring { Cells }
    NormalSpace { Tangent }
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct TexMapping {
    #[serde(rename = "@tex_mapping.rotation")]
    pub rotation: Vec3,
    #[serde(rename = "@tex_mapping.scale")]
    pub scale: Vec3,
    #[serde(rename = "@tex_mapping.translation")]
    pub translation: Vec3,
    #[serde(rename = "@tex_mapping.type")]
    pub mapping_type: TexMappingType,
    #[serde(rename = "@tex_mapping.x_mapping")]
    pub x_mapping: Option<Axis>,
    #[serde(rename = "@tex_mapping.y_mapping")]
    pub y_mapping: Option<Axis>,
    #[serde(rename = "@tex_mapping.z_mapping")]
    pub z_mapping: Option<Axis>,
    #[serde(default, deserialize_with = "py_bool")]
    #[serde(rename = "@tex_mapping.use_minmax")]
    pub use_minmax: Option<bool>,
}

impl TexMapping {
    fn to_python(&self, var: &str) -> Vec<String> {
        let tm = format!("{var}.node.texture_mapping");
        let mut v = vec![
            format!("{tm}.rotation = {}", self.rotation),
            format!("{tm}.scale = {}", self.scale),
            format!("{tm}.translation = {}", self.translation),
            format!("{tm}.vector_type = {}", python_enum(self.mapping_type)),
        ];
        if let Some(x) = self.x_mapping {
            v.push(format!("{tm}.mapping_x = '{x:?}'"));
        }
        if let Some(y) = self.y_mapping {
            v.push(format!("{tm}.mapping_x = '{y:?}'"));
        }
        if let Some(z) = self.z_mapping {
            v.push(format!("{tm}.mapping_x = '{z:?}'"));
        }
        if let Some(b) = self.use_minmax {
            v.push(format!("{tm}.use_minmax = {}", python_bool(b)));
        }
        v
    }
}

#[node]
struct GroupReferenceOutput {
    #[rename = "@type"]
    data_type: SocketType,
}

#[node]
struct GroupReference {
    group_name: String,
    #[serde(default)]
    #[rename = "input"] // another workaround
    inputs_: Vec<GroupReferenceInput>,
    outputs: Vec<GroupReferenceOutput>,
}

impl INode for GroupReference {
    const PYTHON_TYPE: &str = "ShaderNodeGroup";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![(
            "node_tree",
            format!("node_group_{}()", self.group_name.to_snake_case()),
        )]
    }
    fn after(&self) -> Vec<String> {
        self.inputs_
            .iter()
            .filter_map(|input| {
                input.value.map(|value| NodeInput {
                    name: input.name.clone(),
                    value,
                })
            })
            .map(|input| {
                format!(
                    "{}.node.inputs['{}'].default_value = {}",
                    self.name, input.name, input.value
                )
            })
            .collect()
    }
}

#[node]
struct GroupInput {}

impl INode for GroupInput {
    const PYTHON_TYPE: &str = "NodeGroupInput";
}

#[node]
struct GroupOutput {}

impl INode for GroupOutput {
    const PYTHON_TYPE: &str = "NodeGroupOutput";
}

#[node]
struct Bump {
    enable: bool,
    invert: bool,
    inputs: Vec<NodeInput>,
}

fn python_bool(b: bool) -> String {
    match b {
        true => "True".to_owned(),
        false => "False".to_owned(),
    }
}

impl INode for Bump {
    const PYTHON_TYPE: &str = "ShaderNodeBump";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("mute", python_bool(!self.enable)),
            ("invert", python_bool(self.invert)),
        ]
    }
}

#[node]
struct NoiseTexture {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

impl INode for NoiseTexture {
    const PYTHON_TYPE: &str = "ShaderNodeTexNoise";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn after(&self) -> Vec<String> {
        self.tex_mapping.to_python(&self.name)
    }
}

#[node]
struct RoundingEdgeNormal {
    enable: bool,
    inputs: Vec<NodeInput>,
}

impl INode for RoundingEdgeNormal {
    const PYTHON_TYPE: &str = "ShaderNodeBevel";
    fn inputs_override(&self) -> Vec<NodeInput> {
        self.inputs
            .iter()
            .filter(|i| i.name != "Samples")
            .cloned()
            .collect()
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        let mut v = vec![("mute", python_bool(!self.enable))];
        if let Some(i) = self.inputs.iter().find(|i| i.name == "Samples") {
            v.push(("samples", i.value.to_string()));
        }
        v
    }
}

#[node]
struct SwitchClosure {
    enable: bool,
}

impl INode for SwitchClosure {
    const PYTHON_TYPE: &str = "ShaderNodeMixShader";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("mute", python_bool(!self.enable))]
    }
}

#[node]
struct MixClosure {
    inputs: Vec<NodeInput>,
}

impl INode for MixClosure {
    const PYTHON_TYPE: &str = "ShaderNodeMixShader";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct Math {
    #[rename = "@type"]
    operation: MathOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

impl INode for Math {
    const PYTHON_TYPE: &str = "ShaderNodeMath";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("operation", python_enum(self.operation)),
            ("use_clamp", python_bool(self.use_clamp)),
        ]
    }
}

#[node]
struct Mapping {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

impl INode for Mapping {
    const PYTHON_TYPE: &str = "ShaderNodeMapping";
    fn inputs_override(&self) -> Vec<NodeInput> {
        let mut v = self.inputs.clone();
        v.extend([
            NodeInput {
                name: "Location".into(),
                value: NodeInputValue::Vector(self.tex_mapping.translation),
            },
            NodeInput {
                name: "Rotation".into(),
                value: NodeInputValue::Vector(self.tex_mapping.rotation),
            },
            NodeInput {
                name: "Scale".into(),
                value: NodeInputValue::Vector(self.tex_mapping.scale),
            },
        ]);
        v
    }
}

#[node]
struct RgbRamp {
    interpolate: bool,
    #[serde(deserialize_with = "float_seq")]
    ramp: Vec<f32>, // TODO
    #[serde(deserialize_with = "float_seq")]
    ramp_alpha: Vec<f32>, // TODO
}

impl INode for RgbRamp {
    const PYTHON_TYPE: &str = "ShaderNodeValToRGB";
    fn after(&self) -> Vec<String> {
        vec![format!(
            "{}.node.color_ramp.interpolation = '{}'",
            self.name,
            if self.interpolate {
                "LINEAR"
            } else {
                "CONSTANT"
            }
        )]
        // TODO: ramps
    }
}

#[node]
struct DiffuseBsdf {
    inputs: Vec<NodeInput>,
}

impl INode for DiffuseBsdf {
    const PYTHON_TYPE: &str = "ShaderNodeBsdfDiffuse";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct ProjectToAxisPlane {}

impl INode for ProjectToAxisPlane {
    const PYTHON_TYPE: &str = "ShaderNodeGroup";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("node_tree", "node_group_project_to_axis_plane()".to_owned())]
    }
}

#[node]
struct Value {
    value: f32,
}

impl INode for Value {
    const PYTHON_TYPE: &str = "ShaderNodeValue";
    fn after(&self) -> Vec<String> {
        vec![format!(
            "{}.node.outputs[0].default_value = {}",
            self.name, self.value,
        )]
    }
}

#[node]
struct ObjectInfo {}

impl INode for ObjectInfo {
    const PYTHON_TYPE: &str = "ShaderNodeObjectInfo";
}

#[node]
struct ImageTexture {
    color_space: ColorSpace,
    extension: Extension,
    filename: Option<String>,
    interpolation: Interpolation,
    max_mip_lvl: u8,
    projection: Projection,
    #[serde(flatten)]
    tex_mapping: TexMapping,
    texel_per_pixel: f32,
}

impl INode for ImageTexture {
    const PYTHON_TYPE: &str = "ShaderNodeTexImage";
    fn attributes(&self) -> Vec<(&str, String)> {
        let image_name = match &self.filename {
            Some(filename) => format!("{filename:?}"),
            None => "None".to_owned(),
        };
        // TODO: all the other attributes
        vec![("image", format!("load_image({image_name})"))]
    }
}

#[node]
struct MixValue {
    #[rename = "@type"]
    mix_type: MixType,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

impl INode for MixValue {
    const PYTHON_TYPE: &str = "ShaderNodeMix";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("data_type", "'FLOAT'".into()),
            ("blend_type", python_enum(self.mix_type)),
            ("clamp_factor", python_bool(self.use_clamp)),
            ("clamp_result", python_bool(self.use_clamp)),
        ]
    }
}

#[node]
struct SwitchFloat {
    enable: bool,
    inputs: Vec<NodeInput>,
}

impl INode for SwitchFloat {
    const PYTHON_TYPE: &str = "ShaderNodeMix";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("data_type", "'FLOAT'".into()),
            ("mute", python_bool(!self.enable)),
        ]
    }
}

#[node]
struct UvDegradation {
    inputs: Vec<NodeInput>,
}

impl INode for UvDegradation {
    const PYTHON_TYPE: &str = "ShaderNodeGroup";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("node_tree", "node_group_uv_degradation()".to_owned())]
    }
}

#[node]
struct Mix {
    #[rename = "@type"]
    operation: MixOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

impl INode for Mix {
    const PYTHON_TYPE: &str = "ShaderNodeMix";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("data_type", "'RGBA'".into()),
            ("blend_type", python_enum(self.operation)),
        ]
    }
}

#[node]
struct VectorTransform {
    convert_from: VectorSpace,
    convert_to: VectorSpace,
    #[rename = "@type"]
    vector_type: VectorType,
}

impl INode for VectorTransform {
    const PYTHON_TYPE: &str = "ShaderNodeVectorTransform";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("convert_from", python_enum(self.convert_from)),
            ("convert_to", python_enum(self.convert_to)),
            ("vector_type", python_enum(self.vector_type)),
        ]
    }
}

#[node]
struct TextureCoordinate {}

impl INode for TextureCoordinate {
    const PYTHON_TYPE: &str = "ShaderNodeTexCoord";
}

#[node]
struct VectorMath {
    #[rename = "@type"]
    operation: VectorOperation,
    inputs: Vec<NodeInput>,
}

impl INode for VectorMath {
    const PYTHON_TYPE: &str = "ShaderNodeVectorMath";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("operation", python_enum(self.operation))]
    }
}

#[node]
struct PrincipledBsdf {
    distribution: BsdfDistribution,
    subsurface_method: Option<SubsurfaceMethod>,
    inputs: Vec<NodeInput>,
}

impl INode for PrincipledBsdf {
    const PYTHON_TYPE: &str = "ShaderNodeBsdfPrincipled";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        let mut v = vec![("distribution", python_enum(self.distribution))];
        if let Some(m) = self.subsurface_method {
            v.push(("subsurface_method", python_enum(m)));
        }
        v
    }

    fn inputs_override(&self) -> Vec<NodeInput> {
        self.inputs
            .iter()
            .cloned()
            .map(|mut i| {
                if i.name.ends_with("Tint") {
                    if let NodeInputValue::Float(n) = i.value {
                        i.value = NodeInputValue::Color(Vec3([n, n, n]));
                    }
                } else if i.name == "SubsurfaceColor" {
                    if let NodeInputValue::Color(x) = i.value {
                        i.value = NodeInputValue::Vector(x)
                    }
                }
                i
            })
            .collect()
    }
}

#[node]
struct BrightnessContrast {
    inputs: Vec<NodeInput>,
}

impl INode for BrightnessContrast {
    const PYTHON_TYPE: &str = "ShaderNodeBrightContrast";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct NormalMap {
    attribute: String,
    space: NormalSpace,
    inputs: Vec<NodeInput>,
}

impl INode for NormalMap {
    const PYTHON_TYPE: &str = "ShaderNodeNormalMap";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("uv_map", format!("{:?}", self.attribute)),
            ("space", python_enum(self.space)),
        ]
    }
}

#[node]
struct Uvmap {
    attribute: String,
    from_dupli: bool,
}

impl INode for Uvmap {
    const PYTHON_TYPE: &str = "ShaderNodeUVMap";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![
            ("uv_map", format!("{:?}", self.attribute)),
            ("from_instancer", python_bool(self.from_dupli)),
        ]
    }
}

#[node]
struct GlossyBsdf {
    distribution: BsdfDistribution,
    inputs: Vec<NodeInput>,
}

impl INode for GlossyBsdf {
    const PYTHON_TYPE: &str = "ShaderNodeBsdfAnisotropic"; // unsure
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("distribution", python_enum(self.distribution))]
    }
}

#[node]
struct Vector {
    value: Vec3,
}

impl INode for Vector {
    const PYTHON_TYPE: &str = "ShaderNodeCombineXYZ";
    fn after(&self) -> Vec<String> {
        let var = &self.name;
        let [x, y, z] = self.value.0;
        vec![
            format!("{var}.node.inputs[0].default_value = {x}"),
            format!("{var}.node.inputs[1].default_value = {y}"),
            format!("{var}.node.inputs[2].default_value = {z}"),
        ]
    }
}

#[node]
struct RgbCurves {
    #[serde(deserialize_with = "float_seq")]
    #[rename = "@curves"] // suppress the child-element autodetection
    curves: Vec<f32>,
    min_x: f32,
    max_x: f32,
    inputs: Vec<NodeInput>,
}

impl INode for RgbCurves {
    const PYTHON_TYPE: &str = "ShaderNodeRGBCurve";
    // TODO
}

#[node]
struct VoronoiTexture {
    coloring: VoronoiColoring,
    inputs: Vec<NodeInput>,
}

impl INode for VoronoiTexture {
    const PYTHON_TYPE: &str = "ShaderNodeTexVoronoi";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct Geometry {}

impl INode for Geometry {
    const PYTHON_TYPE: &str = "ShaderNodeNewGeometry";
}

#[node]
struct AbsorptionVolume {
    inputs: Vec<NodeInput>,
}

impl INode for AbsorptionVolume {
    const PYTHON_TYPE: &str = "ShaderNodeVolumeAbsorption";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct AddClosure {}

impl INode for AddClosure {
    const PYTHON_TYPE: &str = "ShaderNodeAddShader";
}

#[node]
struct LayerWeight {
    inputs: Vec<NodeInput>,
}

impl INode for LayerWeight {
    const PYTHON_TYPE: &str = "ShaderNodeLayerWeight";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct TranslucentBsdf {
    inputs: Vec<NodeInput>,
}

impl INode for TranslucentBsdf {
    const PYTHON_TYPE: &str = "ShaderNodeBsdfTranslucent";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct TransparentBsdf {
    inputs: Vec<NodeInput>,
}

impl INode for TransparentBsdf {
    const PYTHON_TYPE: &str = "ShaderNodeBsdfTransparent";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

#[node]
struct Color {
    value: Vec3,
}

impl INode for Color {
    const PYTHON_TYPE: &str = "ShaderNodeRGB";
    fn attributes(&self) -> Vec<(&str, String)> {
        vec![("value", self.value.to_string())]
    }
}

#[node]
struct Emission {
    inputs: Vec<NodeInput>,
}

impl INode for Emission {
    const PYTHON_TYPE: &str = "ShaderNodeEmission";
    fn inputs(&self) -> &[NodeInput] {
        &self.inputs
    }
}

macro_rules! nodes {
    ($name:ident $($ty:ident)*) => {
        #[derive(Deserialize, Debug, PartialEq, Clone)]
        #[serde(rename_all = "snake_case")]
        #[enum_dispatch(Named)]
        pub enum Node {
            Group(GroupReference),
            $($ty($ty),)*
        }

        impl INode for Node {
            const PYTHON_TYPE: &str = "";
            fn python_type(&self) -> &'static str {
                match self {
                    Self::Group(x) => x.python_type(),
                    $(Self::$ty(x) => x.python_type(),)*
                }
            }
            fn inputs(&self) -> &[NodeInput] {
                match self {
                    Self::Group(x) => x.inputs(),
                    $(Self::$ty(x) => x.inputs(),)*
                }
            }
            fn inputs_override(&self) -> Vec<NodeInput> {
                match self {
                    Self::Group(x) => x.inputs_override(),
                    $(Self::$ty(x) => x.inputs_override(),)*
                }
            }
            fn attributes(&self) -> Vec<(&str, String)> {
                match self {
                    Self::Group(x) => x.attributes(),
                    $(Self::$ty(x) => x.attributes(),)*
                }
            }
            fn after(&self) -> Vec<String> {
                match self {
                    Self::Group(x) => x.after(),
                    $(Self::$ty(x) => x.after(),)*
                }
            }
        }
    }
}

nodes! {
    Node

    GroupInput GroupOutput Bump NoiseTexture RoundingEdgeNormal SwitchClosure
    MixClosure Math Mapping RgbRamp DiffuseBsdf ProjectToAxisPlane Value
    ObjectInfo ImageTexture MixValue SwitchFloat UvDegradation Mix
    VectorTransform TextureCoordinate VectorMath PrincipledBsdf
    BrightnessContrast NormalMap Uvmap GlossyBsdf Vector RgbCurves
    VoronoiTexture Geometry AbsorptionVolume AddClosure LayerWeight
    TranslucentBsdf TransparentBsdf Color Emission
}
