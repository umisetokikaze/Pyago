use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use crate::config::PyagoToml;
use crate::utils::uninstall_package;

pub fn remove_package(package: Option<&str>, file: Option<&str>) -> Result<()> {
    let pyago_toml_path = Path::new("pyago.toml");
    if !pyago_toml_path.exists() {
        return Err(anyhow!("pyago.tomlファイルが見つかりません。プロジェクトのルートディレクトリで実行してください。"));
    }

    let venv_path = Path::new("venv");
    if !venv_path.exists() {
        return Err(anyhow!("仮想環境が見つかりません。プロジェクトを初期化してください。"));
    }

    let mut pyago_toml: PyagoToml = toml::from_str(&fs::read_to_string(pyago_toml_path)?)?;

    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{spinner:.green} {msg}")
        .unwrap();

    match (package, file) {
        (Some(pkg), None) => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(spinner_style);
            pb.set_message(format!("Removing package: {}", pkg));
            remove_single_package(&mut pyago_toml, venv_path, pkg, &pb)?;
            pb.finish_with_message(format!("Package removed successfully: {}", pkg));
        },
        (None, Some(file_path)) => {
            remove_packages_from_file(&mut pyago_toml, venv_path, file_path)?;
        },
        _ => return Err(anyhow!("パッケージ名またはファイル名のいずれかを指定してください。")),
    }

    fs::write(pyago_toml_path, toml::to_string(&pyago_toml)?)?;

    Ok(())
}

fn remove_single_package(pyago_toml: &mut PyagoToml, venv_path: &Path, package: &str, pb: &ProgressBar) -> Result<()> {
    pb.set_message(format!("Uninstalling package: {}", package));
    uninstall_package(venv_path, package)?;

    pb.set_message(format!("Updating pyago.toml: {}", package));
    pyago_toml.dependencies.remove(package);
    Ok(())
}

fn remove_packages_from_file(pyago_toml: &mut PyagoToml, venv_path: &Path, file_path: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)?;
    let packages: Vec<_> = content.lines().filter(|line| !line.trim().is_empty()).collect();

    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    for package in packages {
        pb.set_message(format!("Removing package: {}", package));
        remove_single_package(pyago_toml, venv_path, package, &pb)?;
        pb.inc(1);
    }

    pb.finish_with_message("All packages removed successfully");
    Ok(())
}
