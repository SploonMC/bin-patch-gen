use proc_macros::{serial, serial_pascal};

#[serial]
pub struct SpigotVersionMeta {
    pub name: String,
    pub description: String,
    pub refs: SpigotVersionRefs,
}

#[serial_pascal]
pub struct SpigotVersionRefs {
    pub build_data: String,
    pub bukkit: String,
    pub craft_bukkit: String,
    pub spigot: String
}

#[serial]
pub struct SpigotBuildData {
    pub server_url: String
}
