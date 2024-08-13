use anyhow::Result;
use crate::utils::{run_command, get_project_root};

pub fn build(optimize: bool, obfuscate: bool, cross_platform: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let main_file = project_root.join("src").join("main.py");

    if !main_file.exists() {
        anyhow::bail!("main.pyファイルが見つかりません");
    }

    let mut build_command = vec!["nuitka3", "--standalone"];

    if optimize {
        build_command.push("--lto=yes");
    }

    if obfuscate {
        build_command.push("--obfuscate");
    }

    if cross_platform {
        // クロスプラットフォームビルドの設定を追加
    }

    build_command.push(main_file.to_str().unwrap());

    run_command(&build_command)?;

    println!("ビルドが完了しました");
    Ok(())
}