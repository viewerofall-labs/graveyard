use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Deserialize, Debug, Clone)]
pub struct Language {
    pub compiler: String,
    pub compiler_var: String,
    pub extension: String,
    pub default_flags: String,
    pub compile_cmd: String,
    pub description: String,
    #[serde(default)]
    pub options: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct LanguageRegistry {
    pub languages: HashMap<String, Language>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BuildConfig {
    pub targets: HashMap<String, Target>,
    #[serde(default)]
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Target {
    #[serde(rename = "type")]
    pub target_type: String,
    pub source: Option<String>,
    pub output: String,
    #[serde(default)]
    pub flags: Option<String>,
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
    #[serde(default)]
    pub link_with: Vec<String>,
}

pub fn load_language_registry(path: &str) -> Result<LanguageRegistry> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn load_build_config(path: &str) -> Result<BuildConfig> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

pub fn transpile_to_makefile(
    config: &BuildConfig,
    registry: &LanguageRegistry,
) -> Result<String> {
    let mut mk = String::new();
    mk.push_str("# Auto-generated Makefile from build.toml\n");
    mk.push_str("# DO NOT EDIT MANUALLY\n");
    mk.push_str("# Regenerate with: build-tool transpile\n\n");

    // Declare all compiler variables
    for (_, lang) in &registry.languages {
        mk.push_str(&format!("{} ?= {}\n", lang.compiler_var, lang.compiler));
    }
    mk.push_str("\n");

    // User variables from build.toml
    if let Some(vars) = &config.variables {
        for (key, val) in vars {
            mk.push_str(&format!("{} = {}\n", key, val));
        }
        mk.push_str("\n");
    }

    // Phony targets
    let target_names: Vec<String> = config.targets.keys().cloned().collect();
    mk.push_str(&format!(".PHONY: all clean help {}\n\n", target_names.join(" ")));

    // Generate rules for each target
    for (name, target) in &config.targets {
        let lang = registry
            .languages
            .get(&target.target_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown language type: '{}' in target '{}'", target.target_type, name))?;

        let source = target.source.as_deref().unwrap_or("unknown");
        let flags = target
            .flags
            .as_deref()
            .unwrap_or(&lang.default_flags);

        // Substitute placeholders in compile_cmd
        let rule = lang
            .compile_cmd
            .replace("{compiler}", &format!("$({})", lang.compiler_var))
            .replace("{flags}", flags)
            .replace("{source}", source)
            .replace("{output}", &target.output);

        // Build prerequisites from source + depends_on
        let mut prerequisites = vec![source.to_string()];
        if let Some(deps) = &target.depends_on {
            prerequisites.extend(deps.clone());
        }
        let prereq_str = prerequisites.join(" ");

        mk.push_str(&format!("{}:\t{}\n", name, prereq_str));
        mk.push_str(&format!("\t{}\n\n", rule));

        // Also create rule for output file if different from target name
        if target.output != *name {
            mk.push_str(&format!("{}:\t{}\n", target.output, prereq_str));
            mk.push_str(&format!("\t{}\n\n", rule));
        }
    }

    // all target
    mk.push_str(&format!("all: {}\n\n", target_names.join(" ")));

    // clean target
    mk.push_str("clean:\n");
    for target in config.targets.values() {
        mk.push_str(&format!("\t@rm -f {}\n", target.output));
    }
    mk.push_str("\t@cargo clean 2>/dev/null || true\n\n");

    // help target
    mk.push_str("help:\n");
    mk.push_str("\t@echo \"Available targets:\"\n");
    for name in &target_names {
        mk.push_str(&format!("\t@echo \"  - {}\"\n", name));
    }

    Ok(mk)
}

pub fn transpile(toml_path: &str, registry_path: &str, output_path: &str) -> Result<()> {
    let config = load_build_config(toml_path)?;
    let registry = load_language_registry(registry_path)?;
    let makefile = transpile_to_makefile(&config, &registry)?;
    fs::write(output_path, makefile)?;
    Ok(())
}
