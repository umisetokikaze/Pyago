use std::fs::{self, File};
use std::path::Path;
use anyhow::Result;
use crate::config::PyagoToml;
use crate::config::Project;
use crate::utils::create_venv;

pub fn init(name: &str) -> Result<()> {
    let path = Path::new(name);
    if path.exists() {
        anyhow::bail!("エラー: {} は既に存在します", name);
    }

    fs::create_dir(path)?;
    fs::create_dir(path.join("src"))?;
    fs::create_dir(path.join("tests"))?;

    let pyago_toml = PyagoToml {
        project: Project {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: "A new Python project".to_string(),
            authors: vec![],
        },
        dependencies: toml::Table::new(),
        dev_dependencies: toml::Table::new(),
    };

    let toml_string = toml::to_string(&pyago_toml)?;
    fs::write(path.join("pyago.toml"), toml_string)?;

    // .gitignoreファイルの内容を書き込む
    fs::write(path.join(".gitignore"), include_str!("../../templates/gitignore.txt"))?;

    // main.pyファイルを作成
    let main_py = format!(
        r#"def main():
    print('Hello from {}!')

if __name__ == '__main__':
    main()
"#,
        name
    );
    fs::write(path.join("src").join("main.py"), main_py)?;

    File::create(path.join("src").join("__init__.py"))?;

    // test_main.pyファイルを作成
    let test_main_py = r#"def test_main():
    assert True
"#;
    fs::write(path.join("tests").join("test_main.py"), test_main_py)?;

    // 仮想環境を作成
    let venv_path = path.join("venv");
    create_venv(&venv_path)?;
    println!("仮想環境を作成しました: {}", venv_path.display());

    println!("{} プロジェクトが正常に作成されました！", name);
    Ok(())
}