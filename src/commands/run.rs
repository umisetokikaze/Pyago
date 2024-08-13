use std::process::Command;
use std::path::Path;
use anyhow::{Result, anyhow};
use crate::utils::DefaultPackageManager;
use crate::utils::PackageManager;
use crate::config::PyagoToml;
pub fn run(command: &str, args: &[String]) -> Result<()> {
    let pyago_toml_path = Path::new("pyago.toml");
    if !pyago_toml_path.exists() {
        return Err(anyhow!("pyago.tomlファイルが見つかりません。プロジェクトのルートディレクトリで実行してください。"));
    }

    let pyago_toml: PyagoToml = toml::from_str(&std::fs::read_to_string(pyago_toml_path)?)?;
    check_and_install_dependencies(&pyago_toml, &Path::new("venv"))?;
    let &mut venv_python;
    if cfg!(windows) {
        venv_python = "venv\\Scripts\\python.exe"
    } else {
        venv_python = "venv/bin/python"
    };

    let mut cmd = Command::new(venv_python);

    if command == "dev" {
        cmd.arg("src/main.py");
    } else {
        cmd.arg("-m").arg(command);
    }

    cmd.args(args);

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("コマンドの実行に失敗しました");
    }

    Ok(())
}

fn check_and_install_dependencies(pyago_toml: &PyagoToml, venv_path: &Path) -> Result<()> {
    let installed_packages = get_installed_packages(venv_path)?;

    for (package, _) in &pyago_toml.dependencies {
        if !installed_packages.contains(package) {
            println!("パッケージ {} がインストールされていません。インストールします...", package);
            let package_manager = DefaultPackageManager::new();
            package_manager.install(venv_path, package)?;
        }
    }

    Ok(())
}

fn get_installed_packages(venv_path: &Path) -> Result<Vec<String>> {

    let pip_path = if cfg!(windows) {
        venv_path.join("Scripts").join("pip.exe")
    } else {
        venv_path.join("bin").join("pip")
    };

    let output = Command::new(pip_path)
        .args(&["list", "--format=freeze"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("インストール済みパッケージの取得に失敗しました"));
    }

    let packages: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.split("==").next().unwrap_or("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

    Ok(packages)
}
