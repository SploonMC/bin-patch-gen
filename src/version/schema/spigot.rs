use proc_macros::{serial, serial_pascal};

#[serial]
pub struct SpigotVersionMeta {
    pub name: String,
    pub description: String,
    pub refs: SpigotVersionRefs,
}

impl SpigotVersionMeta {
    pub fn refs_eq(&self, other: SpigotVersionRefs) -> bool {
        let refs = self.refs.clone();

        refs.build_data == other.build_data
            && refs.bukkit == other.bukkit
            && refs.craft_bukkit == other.craft_bukkit
            && refs.spigot == other.spigot
    }
}

#[serial_pascal]
pub struct SpigotVersionRefs {
    pub build_data: String,
    pub bukkit: String,
    pub craft_bukkit: String,
    pub spigot: String,
}

#[serial]
pub struct SpigotBuildData {
    pub server_url: String,
}
