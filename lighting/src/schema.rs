use std::{fmt::Display, str::FromStr};

use serde::de::{Deserialize, Deserializer, Error, Unexpected};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Eyesight {
    #[serde(rename = "preset")]
    pub presets: Vec<Preset>,
    #[serde(rename = "transform")]
    pub cameras: Vec<Camera>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(default, rename = "transform")]
    pub lights: Vec<Light>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Light {
    #[serde(rename = "@matrix", deserialize_with = "float_arr")]
    pub matrix: [f64; 16],
    #[serde(rename = "light")]
    pub inner: LightInner,
}

#[derive(Deserialize, Debug, Default)]
pub struct LightInner {
    #[serde(rename = "shader")]
    pub inner: LightInnerInner,

    #[serde(rename = "@cast_shadow")]
    pub cast_shadow: bool,

    #[serde(rename = "@cvisibility.diffuse")]
    pub diffuse: bool,
    #[serde(rename = "@cvisibility.glossy")]
    pub glossy: bool,
    #[serde(rename = "@cvisibility.scatter")]
    pub scatter: bool,
    #[serde(rename = "@cvisibility.shadow")]
    pub shadow: bool,
    #[serde(rename = "@cvisibility.transmission")]
    pub transmission: bool,

    #[serde(rename = "@max_bounces")]
    pub max_bounces: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@samples")]
    pub samples: u32,
    #[serde(rename = "@size")]
    pub size: f64,
    #[serde(rename = "@type")]
    pub light_type: LightType,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LightInnerInner {
    pub background_shader: Option<LightNode>,
    pub emission: Option<LightNode>,
    pub connect: Link,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LightNode {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "input")]
    pub inputs: Vec<NodeInput>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Link {
    #[serde(rename = "@from_node")]
    pub from_node: String,
    #[serde(rename = "@from_socket")]
    pub from_socket: String,
    #[serde(rename = "@to_node")]
    pub to_node: String,
    #[serde(rename = "@to_socket")]
    pub to_socket: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type", rename_all = "lowercase")]
pub enum NodeInput {
    Color {
        #[serde(rename = "@name")]
        name: String,
        #[serde(rename = "@value", deserialize_with = "float_arr")]
        value: [f32; 3],
    },
    Float {
        #[serde(rename = "@name")]
        name: String,
        #[serde(rename = "@value", deserialize_with = "deserialize_from_str")]
        value: f32,
    },
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    #[serde(rename = "@matrix", deserialize_with = "float_arr")]
    pub matrix: [f64; 16],
    #[serde(rename = "camera")]
    pub inner: CameraInner,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CameraInner {
    #[serde(rename = "@aperture_ratio")]
    pub aperture_ratio: f32,
    #[serde(rename = "@aperturesize")]
    pub aperture_size: f32,
    #[serde(rename = "@blades")]
    pub blades: u8,
    #[serde(rename = "@bladesrotation_deg")]
    pub blades_rotation: f32,
    #[serde(rename = "@convergence_distance")]
    pub convergence_distance: f64,
    #[serde(rename = "@eyesight.aperture_type")]
    pub aperture_type: ApertureType,
    #[serde(rename = "@eyesight.lens_mm")]
    pub focal_length: f32,
    #[serde(rename = "@eyesight.ortho_scale")]
    pub ortho_scale: f32,
    #[serde(rename = "@eyesight.pixelaspect.x")]
    pub pixel_aspect_x: f32,
    #[serde(rename = "@eyesight.pixelaspect.y")]
    pub pixel_aspect_y: f32,
    #[serde(rename = "@eyesight.shift.x")]
    pub shift_x: f32,
    #[serde(rename = "@eyesight.shift.y")]
    pub shift_y: f32,
    #[serde(rename = "@farclip")]
    pub far_clip: f32,
    #[serde(rename = "@focaldistance")]
    pub focal_distance: f32,
    #[serde(rename = "@interocular_distance")]
    pub interocular_distance: f64,
    #[serde(rename = "@motion_position")]
    pub motion_position: MotionPosition,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@nearclip")]
    pub near_clip: f64,
    #[serde(rename = "@resolution_percentage")]
    pub resolution_percentage: u8,
    #[serde(rename = "@resolution_x")]
    pub resolution_x: u32,
    #[serde(rename = "@resolution_y")]
    pub resolution_y: u32,
    #[serde(rename = "@rolling_shutter_duration")]
    pub rolling_shutter_duration: f64,
    #[serde(rename = "@rolling_shutter_type")]
    pub rolling_shutter_type: RollingShutterType,
    #[serde(rename = "@sensor_fit")]
    pub sensor_fit: SensorFit,
    #[serde(rename = "@sensor_size_mm")]
    pub sensor_size: f32,
    #[serde(rename = "@shutter_curve", deserialize_with = "float_arr")]
    pub shutter_curve: [f32; 256],
    #[serde(rename = "@shuttertime")]
    pub shutter_time: f32,
    #[serde(rename = "@type")]
    pub camera_type: CameraType,
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
            #[derive(Deserialize, Debug)]
            #[serde(rename_all = "lowercase")]
            pub enum $enum {
                $($variant),*
            }
        )*
    }
}

enums! {
    LightType { Point, Distant, Background, Area, Spot, Triangle, Ambient }
    ApertureType { Radius, FStop }
    MotionPosition { Start, Center, End }
    RollingShutterType { None, Top }
    SensorFit { Horizontal, Vertical }
    CameraType { Perspective, Orthographic, Panorama }
}

impl Default for LightType {
    fn default() -> Self {
        Self::Distant
    }
}

fn float_arr<'de, D: Deserializer<'de>, T: Default + FromStr, const N: usize>(
    de: D,
) -> Result<[T; N], D::Error> {
    let s: &'de str = Deserialize::deserialize(de)?;
    let n = s.split_ascii_whitespace().count();
    if n != N {
        return Err(D::Error::invalid_length(n, &&*format!("exactly {N} items")));
    }

    let mut arr = std::array::from_fn::<T, N, _>(|_| T::default());
    for (i, word) in s.split_ascii_whitespace().enumerate() {
        arr[i] = word.parse().map_err(|_| {
            D::Error::invalid_value(Unexpected::Str(word), &"a floating-point number")
        })?;
    }
    Ok(arr)
}

fn deserialize_from_str<'de, D: Deserializer<'de>, T: FromStr>(de: D) -> Result<T, D::Error>
where
    T::Err: Display,
{
    let s: &'de str = Deserialize::deserialize(de)?;
    s.parse().map_err(|e| D::Error::custom(e))
}
