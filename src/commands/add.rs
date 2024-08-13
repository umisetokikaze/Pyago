use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{info, warn, error};
use crate::config::{Config};
use crate::utils::{PackageManager, DefaultPackageManager};

pub struct AddCommand {
    packages: Vec<String>,
    requirements_file: Option<String>,
    package_manager: Box<dyn PackageManager>,
}

impl AddCommand {
    pub fn new(packages: Vec<String>, requirements_file: Option<String>) -> Self {
        Self {
            packages,
            requirements_file,
            package_manager: Box::new(DefaultPackageManager::new()),
        }
    }

    pub fn execute(&mut self, _config: &mut Config) -> Result<()> {
        let mut config = self.load_config()?;
        let venv_path = self.get_venv_path(&mut config)?;

        if let Some(file_path) = &self.requirements_file {
            self.add_packages_from_file(&mut config, &venv_path, file_path)
        } else {
            self.add_packages(&mut config, &venv_path)
        }
    }

    fn load_config(&self) -> Result<Config> {
        let current_dir = std::env::current_dir()?;
        let pyago_toml_path = current_dir.join("pyago.toml");
        Config::from_file(&pyago_toml_path)
            .context("Failed to load pyago.toml")
    }

    fn get_venv_path(&self, config: &mut Config) -> Result<PathBuf> {
        let venv_path = config.venv_path()?;
        if !venv_path.exists() {
            error!("Virtual environment not found at {:?}", venv_path);
            anyhow::bail!("Virtual environment not found. Please initialize the project first.");
        }
        Ok(venv_path)
    }

    fn add_packages(&self, config: &mut Config, venv_path: &Path) -> Result<()> {
        let multi_progress = MultiProgress::new();
        let results: Vec<Result<()>> = self.packages.iter()
            .map(|pkg| self.add_single_package(config, venv_path, pkg, multi_progress.clone()))
            .collect();

        multi_progress.clear()?;

        for result in results {
            result?;
        }

        self.save_config(&config)?;
        Ok(())
    }

    fn add_packages_from_file(&self, config: &mut Config, venv_path: &Path, file_path: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .context(format!("Failed to read requirements file: {}", file_path))?;
        let packages: Vec<_> = content.lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        let multi_progress = MultiProgress::new();
        let pb = multi_progress.add(ProgressBar::new(packages.len() as u64));
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        for package in packages {
            pb.set_message(format!("Adding package: {}", package));
            self.add_single_package(config, venv_path, package, multi_progress.clone())?;
            pb.inc(1);
        }

        pb.finish_with_message("All packages added successfully");
        self.save_config(&config)?;
        Ok(())
    }

    fn add_single_package(&self, config: &mut Config, venv_path: &Path, package: &str, multi_progress: MultiProgress) -> Result<()> {
        let pb = multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap());

        if self.package_manager.is_installed(venv_path, package)? {
            warn!("Package already installed: {}", package);
            pb.set_message(format!("Package already installed: {}", package));
            let version = self.package_manager.get_version(venv_path, package)?;
            config.add_dependency(package, &version);
            pb.finish_with_message(format!("Package already installed: {} ({})", package, version));
            return Ok(());
        }

        pb.set_message(format!("Installing package: {}", package));
        self.package_manager.install(venv_path, package)?;

        pb.set_message(format!("Getting package version: {}", package));
        let version = self.package_manager.get_version(venv_path, package)?;

        pb.set_message(format!("Updating pyago.toml: {}", package));
        config.add_dependency(package, &version);

        pb.finish_with_message(format!("Package added successfully: {} ({})", package, version));
        info!("Package added: {} ({})", package, version);
        Ok(())
    }

    fn save_config(&self, config: &Config) -> Result<()> {
        config.save()
            .context("Failed to save pyago.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        PackageManager {}
        impl PackageManager for PackageManager {
            fn is_installed(&self, venv_path: &Path, package: &str) -> Result<bool>;
            fn install(&self, venv_path: &Path, package: &str) -> Result<()>;
            fn get_version(&self, venv_path: &Path, package: &str) -> Result<String>;
        }
    }

    #[test]
    fn test_add_single_package() {
        let mut mock_pm = MockPackageManager::new();
        mock_pm.expect_is_installed()
            .with(eq(Path::new("/venv")), eq("test-package"))
            .return_const(Ok(false));
        mock_pm.expect_install()
            .with(eq(Path::new("/venv")), eq("test-package"))
            .return_const(Ok(()));
        mock_pm.expect_get_version()
            .with(eq(Path::new("/venv")), eq("test-package"))
            .return_const(Ok("1.0.0".to_string()));

        let mut config = Config::default();
        let multi_progress = MultiProgress::new();
        let cmd = AddCommand {
            packages: vec!["test-package".to_string()],
            requirements_file: None,
            package_manager: Box::new(mock_pm),
        };

        let result = cmd.add_single_package(&mut config, Path::new("/venv"), "test-package", &multi_progress);
        assert!(result.is_ok());
        assert_eq!(config.dependencies.get("test-package"), Some(&"1.0.0".to_string()));
    }
}
