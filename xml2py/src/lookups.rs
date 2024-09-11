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
    }
};
