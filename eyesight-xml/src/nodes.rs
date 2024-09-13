use std::borrow::Cow;

use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};

use xml2py_macros::node;

use crate::schema::Named;

pub trait INode: Named {
    const PYTHON_TYPE: &str;
    fn python_type(&self) -> &'static str {
        Self::PYTHON_TYPE
    }
    fn inputs(&self) -> &[NodeInput] {
        &[]
    }
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
            Self::Vector(Vec3([x, y, z])) => write!(f, "({x}, {y}, {z})"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Color(Vec3([r, g, b])) => write!(f, "({r}, {g}, {b})"),
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

macro_rules! enums {
    (
        $(
            $enum:ident {
                $($variant:ident),*$(,)?
            }
        )*
    ) => {
        $(
            #[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
            #[serde(rename_all = "snake_case")]
            pub enum $enum {
                $($variant),*
            }
        )*
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
    VectorOperation { Average }
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

#[node]
struct GroupReferenceOutput {
    #[rename = "@type"]
    data_type: SocketType,
}

#[node(ShaderNodeGroup)]
struct GroupReference {
    group_name: String,
    #[serde(default)]
    #[rename = "input"] // another workaround
    inputs_: Vec<GroupReferenceInput>,
    outputs: Vec<GroupReferenceOutput>,
}

#[node(NodeGroupInput)]
struct GroupInput {}

#[node(NodeGroupOutput)]
struct GroupOutput {}

#[node(ShaderNodeBump)]
struct Bump {
    enable: bool,
    invert: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeTexNoise)]
struct NoiseTexture {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeBevel)]
struct RoundingEdgeNormal {
    enable: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeMix)]
struct SwitchClosure {
    enable: bool,
}

#[node(ShaderNodeMix)]
struct MixClosure {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeMath)]
struct Math {
    #[rename = "@type"]
    operation: MathOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeMapping)]
struct Mapping {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeValToRGB)]
struct RgbRamp {
    interpolate: bool,
    #[serde(deserialize_with = "float_seq")]
    ramp: Vec<f32>, // TODO
    #[serde(deserialize_with = "float_seq")]
    ramp_alpha: Vec<f32>, // TODO
}

#[node(ShaderNodeBsdfDiffuse)]
struct DiffuseBsdf {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeGroup)]
struct ProjectToAxisPlane {}

#[node(ShaderNodeValue)]
struct Value {
    value: f32,
}

#[node(ShaderNodeObjectInfo)]
struct ObjectInfo {}

#[node(ShaderNodeTexImage)]
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

#[node(ShaderNodeMix)]
struct MixValue {
    #[rename = "@type"]
    mix_type: MixType,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeMix)]
struct SwitchFloat {
    enable: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeGroup)]
struct UvDegradation {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeMix)]
struct Mix {
    #[rename = "@type"]
    operation: MixOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeVectorTransform)]
struct VectorTransform {
    convert_from: VectorSpace,
    convert_to: VectorSpace,
    #[rename = "@type"]
    vector_type: VectorType,
}

#[node(ShaderNodeTexCoord)]
struct TextureCoordinate {}

#[node(ShaderNodeVectorMath)]
struct VectorMath {
    #[rename = "@type"]
    operation: VectorOperation,
}

#[node(ShaderNodeBsdfPrincipled)]
struct PrincipledBsdf {
    distribution: BsdfDistribution,
    subsurface_method: Option<SubsurfaceMethod>,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeBrightContrast)]
struct BrightnessContrast {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeNormalMap)]
struct NormalMap {
    attribute: String,
    space: NormalSpace,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeUVMap)]
struct Uvmap {
    attribute: String,
    from_dupli: bool,
}

#[node(ShaderNodeBsdfAnisotropic)] // unsure
struct GlossyBsdf {
    distribution: BsdfDistribution,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeRGB)] // TODO: Custom node group with a three-number panel or whatever
struct Vector {
    value: Vec3,
}

#[node(ShaderNodeRGBCurve)]
struct RgbCurves {
    #[serde(deserialize_with = "float_seq")]
    #[rename = "@curves"] // suppress the child-element autodetection
    curves: Vec<f32>,
    min_x: f32,
    max_x: f32,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeTexVoronoi)]
struct VoronoiTexture {
    coloring: VoronoiColoring,
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeNewGeometry)]
struct Geometry {}

#[node(ShaderNodeVolumeAbsorption)]
struct AbsorptionVolume {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeAddShader)]
struct AddClosure {}

#[node(ShaderNode)]
struct LayerWeight {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeBsdfTranslucent)]
struct TranslucentBsdf {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeBsdfTransparent)]
struct TransparentBsdf {
    inputs: Vec<NodeInput>,
}

#[node(ShaderNodeRGB)]
struct Color {
    value: Vec3,
}

#[node(ShaderNodeEmission)]
struct Emission {
    inputs: Vec<NodeInput>,
}

macro_rules! nodes {
    ($name:ident $($ty:ident)*) => {
        #[derive(Deserialize, Debug, PartialEq, Clone)]
        #[serde(rename_all = "snake_case")]
        pub enum Node {
            Group(GroupReference),
            $($ty($ty),)*
        }

        impl Named for Node {
            fn name(&self) -> &str {
                match self {
                    Self::Group(x) => &x.name,
                    $(Self::$ty(x) => &x.name,)*
                }
            }

            fn name_mut(&mut self) -> &mut String {
                match self {
                    Self::Group(x) => &mut x.name,
                    $(Self::$ty(x) => &mut x.name,)*
                }
            }
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
