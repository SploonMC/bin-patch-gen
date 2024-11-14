use proc_macros::{serial, serial_pascal};

#[serial]
pub struct SpigotVersionMeta {
    pub name: String,
    pub description: String,
    pub refs: SpigotVersionRefs,
    pub tools_version: u16,
}

#[serial_pascal]
pub struct SpigotVersionRefs {
    pub build_data: String,
    pub bukkit: String,
    pub craft_bukkit: String,
    pub spigot: String
}
