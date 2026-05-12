use mlua::prelude::*;
use shared::Group;
use std::path::PathBuf;
use tracing::{info, warn};

pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    pub fn new() -> anyhow::Result<Self> {
        let lua = Lua::new();

        let log_fn = lua
            .create_function(|_, msg: String| {
                info!("[lua] {}", msg);
                Ok(())
            })
            .map_err(|e| anyhow::anyhow!("Failed to create log function: {}", e))?;

        lua.globals()
            .set("log", log_fn)
            .map_err(|e| anyhow::anyhow!("Failed to set log global: {}", e))?;

        Ok(Self { lua })
    }

    pub fn load_config(&self) -> anyhow::Result<()> {
        let config_path = config_path();

        if !config_path.exists() {
            warn!("No config found at {:?}, skipping", config_path);
            return Ok(());
        }

        let script = std::fs::read_to_string(&config_path)?;

        self.lua
            .load(&script)
            .exec()
            .map_err(|e| anyhow::anyhow!("Lua script error: {}", e))?;

        info!("Loaded Lua config from {:?}", config_path);

        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<LuaFunction>("on_startup") {
            func.call::<()>(())
                .map_err(|e| anyhow::anyhow!("on_startup error: {}", e))?;
        }

        Ok(())
    }

    pub fn is_hidden(&self, name: &str) -> bool {
        let globals = self.lua.globals();
        let hidden: LuaTable = match globals.get("hidden_services") {
            Ok(t) => t,
            Err(_) => return false,
        };

        for pair in hidden.sequence_values::<String>() {
            if let Ok(hidden_name) = pair {
                if hidden_name == name {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_groups(&self) -> Vec<Group> {
        let globals = self.lua.globals();
        let table: LuaTable = match globals.get("groups") {
            Ok(t) => t,
            Err(_) => return vec![],
        };

        let mut groups = vec![];

        for pair in table.sequence_values::<LuaTable>() {
            let group_table = match pair {
                Ok(t) => t,
                Err(_) => continue,
            };

            let name: String = match group_table.get("name") {
                Ok(n) => n,
                Err(_) => continue,
            };

            let services_table: LuaTable = match group_table.get("services") {
                Ok(t) => t,
                Err(_) => continue,
            };

            let services: Vec<String> = services_table
                .sequence_values::<String>()
                .filter_map(|s| s.ok())
                .collect();

            groups.push(Group { name, services });
        }

        groups
    }

    pub fn on_service_failed(&self, name: &str) {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<LuaFunction>("on_service_failed") {
            if let Err(e) = func.call::<()>(name.to_string()) {
                warn!("Lua on_service_failed error: {}", e);
            }
        }
    }
}

fn config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config/systemd-tui/config.lua");
    path
}
