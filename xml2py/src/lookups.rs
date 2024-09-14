use phf::phf_map;

pub static INPUT_ALIASES: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = phf_map! {
    "ShaderNodeBsdfPrincipled" => phf_map! {
        "Subsurface" => "Subsurface Weight",
        "Clearcoat" => "Coat Weight",
        "ClearcoatRoughness" => "Coat Roughness",
        "Clearcoat Roughness" => "Coat Roughness",
        "ClearcoatNormal" => "Coat Normal",
        "Clearcoat Normal" => "Coat Normal",
        "Transmission" => "Transmission Weight",
        "Sheen" => "Sheen Weight",
        "SheenTint" => "Sheen Tint",
        "Specular" => "Specular IOR Level",
        "SpecularTint" => "Specular Tint",
        "AnisotropicRotation" => "Anisotropic Rotation",
        "SubsurfaceRadius" => "Subsurface Radius",

        // Not sure these are accurate.
        "TransmissionRoughness" => "Roughness",
        "Transmission Roughness" => "Roughness",
        "SubsurfaceColor" => "Subsurface Radius",
        "BaseColor" => "Base Color",
        "Color" => "Base Color",
    },
    "ShaderNodeMapRange" => phf_map! { "Value" => "Result" },
    "ShaderNodeBevel" => phf_map! { "Size" => "Radius" },
    "ShaderNodeMath" => phf_map! { "Value1" => "0", "Value2" => "1" },
    "ShaderNodeVectorMath" => phf_map! { "Vector1" => "0", "Vector2" => "1" },
    "ShaderNodeAddShader" => phf_map! { "Shader1" => "0", "Shader2" => "1" },
    "ShaderNodeMixShader" => phf_map! { "Shader1" => "1", "Shader2" => "2" },
    "ShaderNodeMix" => phf_map! {
        "Fac" => "0",
        // <switch_float>
        "ValueDisable" => "2",
        "ValueEnable" => "3",
        // <mix_value>
        "Value1" => "2",
        "Value2" => "3",
        // <mix>
        "Color1" => "6",
        "Color2" => "7",
    },
};
