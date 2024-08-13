use std::path::Path;
use anyhow::{Result,Context, anyhow};
use std::path::PathBuf;
use std::process::Command;

pub trait PackageManager {
    fn is_installed(&self, venv_path: &Path, package: &str) -> Result<bool>;
    fn install(&self, venv_path: &Path, package: &str) -> Result<()>;
    fn get_version(&self, venv_path: &Path, package: &str) -> Result<String>;
}

pub struct DefaultPackageManager;

impl DefaultPackageManager {
    pub fn new() -> Self {
        DefaultPackageManager
    }
}

pub fn create_venv(path: &Path) -> Result<()> {
    let output = Command::new("python")
        .args(&["-m", "venv", path.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("仮想環境の作成に失敗しました: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}



pub fn uninstall_package(venv_path: &Path, package: &str) -> Result<()> {
    let pip_path = if cfg!(windows) {
        venv_path.join("Scripts").join("pip.exe")
    } else {
        venv_path.join("bin").join("pip")
    };

    let output = Command::new(pip_path)
        .args(&["uninstall", "-y", package])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("パッケージのアンインストールに失敗しました: {}", package));
    }

    Ok(())
}

pub fn run_command(args: &[&str]) -> Result<()> {
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()?;

    if !status.success() {
        anyhow::bail!("コマンドの実行に失敗しました: {:?}", args);
    }

    Ok(())
}

pub fn get_project_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let pyago_toml = current_dir.join("pyago.toml");

    if pyago_toml.exists() {
        Ok(current_dir)
    } else {
        anyhow::bail!("プロジェクトのルートディレクトリが見つかりません")
    }
}


impl PackageManager for DefaultPackageManager {
    fn is_installed(&self, venv_path: &Path, package: &str) -> Result<bool> {
        let output = Command::new(venv_path.join("bin/pip"))
            .args(&["show", package])
            .output()
            .context("Failed to execute pip show command")?;
        Ok(output.status.success())
    }

    fn install(&self, venv_path: &Path, package: &str) -> Result<()> {
        let status = Command::new(venv_path.join("bin/pip"))
            .args(&["install", package])
            .status()
            .context("Failed to execute pip install command")?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("Failed to install package: {}", package)
        }
    }

    fn get_version(&self, venv_path: &Path, package: &str) -> Result<String> {
        let output = Command::new(venv_path.join("bin/pip"))
            .args(&["show", package])
            .output()
            .context("Failed to execute pip show command")?;
        
        if output.status.success() {
            let output_str = String::from_utf8(output.stdout)
                .context("Failed to parse pip show output")?;
            for line in output_str.lines() {
                if line.starts_with("Version:") {
                    return Ok(line.split_whitespace().nth(1).unwrap_or("").to_string());
                }
            }
            anyhow::bail!("Version information not found for package: {}", package)
        } else {
            anyhow::bail!("Failed to get version for package: {}", package)
        }
    }
}
