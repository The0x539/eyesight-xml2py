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
        "Fac" => "Factor",
        // <switch_float>
        "ValueDisable" => "A",
        "ValueEnable" => "B",
        // <mix_value>
        "Value1" => "A",
        "Value2" => "B",
        // <mix>
        "Color1" => "A",
        "Color2" => "B",
    },
};

pub static OUTPUT_ALIASES: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = phf_map! {
    "ShaderNodeMix" => phf_map! {
        "Value" => "Result",
        "ValueOut" => "Result",
        "Color" => "Result",
    },
    "ShaderNodeTexVoronoi" => phf_map!{
        "Fac" => "Distance",
    },
    "ShaderNodeBrightContrast" => phf_map! {
        "OutColor" => "Color",
    }
};
