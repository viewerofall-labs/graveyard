use std::path::PathBuf;

/// Every asset baked into the binary. On first run these get written out
/// to ~/.cache/greggame/<rel> so Bevy's AssetServer can load them.
const EMBEDDED: &[(&str, &[u8])] = &[
    (
        "models/body/body_light.gltf",
        include_bytes!("../assets/models/body/body_light.gltf"),
    ),
    (
        "models/body/body_dark.gltf",
        include_bytes!("../assets/models/body/body_dark.gltf"),
    ),
    (
        "models/body/body.bin",
        include_bytes!("../assets/models/body/body.bin"),
    ),
    (
        "models/body/T_Eye_Brown.png",
        include_bytes!("../assets/models/body/T_Eye_Brown.png"),
    ),
    (
        "models/body/T_Eye_Normal_png.png",
        include_bytes!("../assets/models/body/T_Eye_Normal_png.png"),
    ),
    (
        "models/body/T_Hair_1_BaseColor.png",
        include_bytes!("../assets/models/body/T_Hair_1_BaseColor.png"),
    ),
    (
        "models/body/T_Hair_1_Normal_png.png",
        include_bytes!("../assets/models/body/T_Hair_1_Normal_png.png"),
    ),
    (
        "models/body/T_Superhero_Male_Normal.png",
        include_bytes!("../assets/models/body/T_Superhero_Male_Normal.png"),
    ),
    (
        "models/body/T_Superhero_Male_Roughness.png",
        include_bytes!("../assets/models/body/T_Superhero_Male_Roughness.png"),
    ),
    (
        "models/body/T_Superhero_Male_Light.png",
        include_bytes!("../assets/models/body/T_Superhero_Male_Light.png"),
    ),
    (
        "models/body/T_Superhero_Male_Dark.png",
        include_bytes!("../assets/models/body/T_Superhero_Male_Dark.png"),
    ),
    (
        "models/hair/Hair_Long.gltf",
        include_bytes!("../assets/models/hair/Hair_Long.gltf"),
    ),
    (
        "models/hair/Hair_Long.bin",
        include_bytes!("../assets/models/hair/Hair_Long.bin"),
    ),
    (
        "models/hair/T_Hair_2_BaseColor.png",
        include_bytes!("../assets/models/hair/T_Hair_2_BaseColor.png"),
    ),
    (
        "models/hair/T_Hair_2_Normal.png",
        include_bytes!("../assets/models/hair/T_Hair_2_Normal.png"),
    ),
];

pub fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.cache", h)))
        .unwrap_or_else(|| ".cache".to_string());
    PathBuf::from(base).join("greggame")
}

/// Extract every embedded asset to the cache dir if it's missing.
pub fn ensure_cache_seeded() -> PathBuf {
    let cache = cache_dir();
    let _ = std::fs::create_dir_all(&cache);
    for (rel, bytes) in EMBEDDED {
        let path = cache.join(rel);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, *bytes);
    }
    cache
}
