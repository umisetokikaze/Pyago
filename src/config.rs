use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::fs;
use toml;


#[derive(Serialize, Deserialize)]
pub struct PyagoToml {
    pub project: Project,
    pub dependencies: toml::Table,
    pub dev_dependencies: toml::Table,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
}



pub struct Config {
    pub dependencies: HashMap<String, String>,
    file_path: PathBuf,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read pyago.toml file")?;
        let pyago_toml: toml::Value = toml::from_str(&content)
            .context("Failed to parse pyago.toml file")?;
        
        let dependencies = pyago_toml.get("dependencies")
            .and_then(|d| d.as_table())
            .map(|t| t.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect())
            .unwrap_or_default();

        Ok(Config {
            dependencies,
            file_path: path.to_path_buf(),
        })
    }

    pub fn venv_path(&self) -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join("venv"))
    }

    pub fn add_dependency(&mut self, package: &str, version: &str) {
        self.dependencies.insert(package.to_string(), version.to_string());
    }

    pub fn save(&self) -> Result<()> {
        let mut toml_value = toml::Value::Table(toml::value::Table::new());
        let mut dependencies = toml::value::Table::new();
        
        for (package, version) in &self.dependencies {
            dependencies.insert(package.clone(), toml::Value::String(version.clone()));
        }
        
        toml_value.as_table_mut().unwrap().insert("dependencies".to_string(), toml::Value::Table(dependencies));
        
        let toml_string = toml::to_string(&toml_value)
            .context("Failed to serialize config to TOML")?;
        
        fs::write(&self.file_path, toml_string)
            .context("Failed to write config to file")?;
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dependencies: HashMap::new(),
            file_path: PathBuf::from("pyago.toml"),
        }
    }
}
