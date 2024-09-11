use std::borrow::Cow;

use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use xml2py_macros::node;

#[node]
struct NodeInput {
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

impl<'de> serde::de::DeserializeSeed<'de> for NodeInputType {
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
    pub data_type: NodeInputType,
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
                let mut data_type = None::<NodeInputType>;
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
    NodeInputType {
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
    data_type: NodeInputType,
}

#[node]
struct GroupReference {
    group_name: String,
    inputs: Vec<GroupReferenceInput>,
    outputs: Vec<GroupReferenceOutput>,
}

#[node]
struct GroupInput {}

#[node]
struct GroupOutput {}

#[node]
struct Bump {
    enable: bool,
    invert: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct NoiseTexture {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

#[node]
struct RoundingEdgeNormal {
    enable: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct SwitchClosure {
    enable: bool,
}

#[node]
struct MixClosure {
    inputs: Vec<NodeInput>,
}

#[node]
struct Math {
    #[rename = "@type"]
    operation: MathOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct Mapping {
    #[serde(flatten)]
    tex_mapping: TexMapping,
    inputs: Vec<NodeInput>,
}

#[node]
struct RgbRamp {
    interpolate: bool,
    #[serde(deserialize_with = "float_seq")]
    ramp: Vec<f32>, // TODO
    #[serde(deserialize_with = "float_seq")]
    ramp_alpha: Vec<f32>, // TODO
}

#[node]
struct DiffuseBsdf {
    inputs: Vec<NodeInput>,
}

#[node]
struct ProjectToAxisPlane {}

#[node]
struct Value {
    value: f32,
}

#[node]
struct ObjectInfo {}

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

#[node]
struct MixValue {
    #[rename = "@type"]
    mix_type: MixType,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct SwitchFloat {
    enable: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct UvDegradation {
    inputs: Vec<NodeInput>,
}

#[node]
struct Mix {
    #[rename = "@type"]
    operation: MixOperation,
    use_clamp: bool,
    inputs: Vec<NodeInput>,
}

#[node]
struct VectorTransform {
    convert_from: VectorSpace,
    convert_to: VectorSpace,
    #[rename = "@type"]
    vector_type: VectorType,
}

#[node]
struct TextureCoordinate {}

#[node]
struct VectorMath {
    #[rename = "@type"]
    operation: VectorOperation,
}

#[node]
struct PrincipledBsdf {
    distribution: BsdfDistribution,
    subsurface_method: Option<SubsurfaceMethod>,
    inputs: Vec<NodeInput>,
}

#[node]
struct BrightnessContrast {
    inputs: Vec<NodeInput>,
}

#[node]
struct NormalMap {
    attribute: String,
    space: NormalSpace,
    inputs: Vec<NodeInput>,
}

#[node]
struct Uvmap {
    attribute: String,
    from_dupli: bool,
}

#[node]
struct GlossyBsdf {
    distribution: BsdfDistribution,
    inputs: Vec<NodeInput>,
}

#[node]
struct Vector {
    value: Vec3,
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

#[node]
struct VoronoiTexture {
    coloring: VoronoiColoring,
    inputs: Vec<NodeInput>,
}

#[node]
struct Geometry {}

#[node]
struct AbsorptionVolume {
    inputs: Vec<NodeInput>,
}

#[node]
struct AddClosure {}

#[node]
struct LayerWeight {
    inputs: Vec<NodeInput>,
}

#[node]
struct TranslucentBsdf {
    inputs: Vec<NodeInput>,
}

#[node]
struct TransparentBsdf {
    inputs: Vec<NodeInput>,
}

#[node]
struct Color {
    value: Vec3,
}

#[node]
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

        impl Node {
            pub fn name(&self) -> &str {
                match self {
                    Self::Group(x) => &x.name,
                    $(Self::$ty(x) => &x.name,)*
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
