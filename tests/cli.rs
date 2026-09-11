use std::{fs, path::Path, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn goto(config: &Path, current_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_goto"))
        .args(args)
        .env("GOTO_CONFIG", config)
        .current_dir(current_dir)
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn supports_every_save_shorthand_and_management_alias() {
    let temp = tempdir().unwrap();
    let config = temp.path().join("data/workplaces.json");
    let explicit = temp.path().join("Explicit");
    let inferred = temp.path().join("Inferred");
    let current = temp.path().join("Current");
    fs::create_dir(&explicit).unwrap();
    fs::create_dir(&inferred).unwrap();
    fs::create_dir(&current).unwrap();

    let output = goto(
        &config,
        temp.path(),
        &["-s", explicit.to_str().unwrap(), "--name", "Named"],
    );
    assert!(output.status.success(), "{:?}", output);

    let output = goto(&config, &current, &["-s", "--name", "CurrentAlias"]);
    assert!(output.status.success(), "{:?}", output);

    let output = goto(&config, temp.path(), &["-s", inferred.to_str().unwrap()]);
    assert!(output.status.success(), "{:?}", output);

    let output = goto(&config, &current, &["-s"]);
    assert!(output.status.success(), "{:?}", output);

    let output = goto(
        &config,
        temp.path(),
        &["save", explicit.to_str().unwrap(), "--name", "Subcommand"],
    );
    assert!(output.status.success(), "{:?}", output);

    let value: Value = serde_json::from_slice(&fs::read(&config).unwrap()).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 5);
    assert!(value.get("Named").is_some());
    assert!(value.get("CurrentAlias").is_some());
    assert!(value.get("Inferred").is_some());
    assert!(value.get("Current").is_some());
    assert!(value.get("Subcommand").is_some());

    let output = goto(&config, temp.path(), &["list"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("CurrentAlias"));

    let output = goto(&config, temp.path(), &["-r", "named"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("Removed Named"));
}

#[test]
fn resolves_raw_key_and_rejects_duplicate_without_modifying_json() {
    let temp = tempdir().unwrap();
    let config = temp.path().join("workplaces.json");
    let target = temp.path().join("folder with spaces");
    fs::create_dir(&target).unwrap();

    assert!(goto(
        &config,
        temp.path(),
        &["-s", target.to_str().unwrap(), "--name", "Work"]
    )
    .status
    .success());
    let before = fs::read(&config).unwrap();

    let resolved = goto(&config, temp.path(), &["work"]);
    assert!(resolved.status.success());
    assert_eq!(stdout(&resolved).trim(), target.display().to_string());

    let duplicate = goto(
        &config,
        temp.path(),
        &["save", target.to_str().unwrap(), "--name", "WORK"],
    );
    assert!(!duplicate.status.success());
    assert!(String::from_utf8(duplicate.stderr)
        .unwrap()
        .contains("already exists"));
    assert_eq!(fs::read(config).unwrap(), before);
}

#[test]
fn double_dash_allows_a_key_matching_a_subcommand() {
    let temp = tempdir().unwrap();
    let config = temp.path().join("workplaces.json");
    let target = temp.path().join("target");
    fs::create_dir(&target).unwrap();

    assert!(goto(
        &config,
        temp.path(),
        &["-s", target.to_str().unwrap(), "--name", "list"]
    )
    .status
    .success());
    let output = goto(&config, temp.path(), &["--", "list"]);
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(stdout(&output).trim(), target.display().to_string());
}

#[test]
fn init_does_not_require_a_configuration_directory() {
    let temp = tempdir().unwrap();
    let output = goto(
        &temp.path().join("missing/config.json"),
        temp.path(),
        &["init", "powershell"],
    );
    assert!(output.status.success());
    assert!(stdout(&output).contains("Set-Location"));
    assert!(!temp.path().join("missing").exists());
}
