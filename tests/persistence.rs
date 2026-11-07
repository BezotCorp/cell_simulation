use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(arguments: &[&str], load: Option<&Path>, save: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cell_simulation"));
    command
        .args(arguments)
        .arg("--headless")
        .arg("--save")
        .arg(save);
    if let Some(load) = load {
        command.arg("--load").arg(load);
    }
    command.output().unwrap()
}
fn field(output: &Output, key: &str) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&format!("{key}=")).map(str::to_owned))
        .unwrap()
}

#[test]
fn command_line_resumes_the_same_experiment_and_random_state() {
    let directory = std::env::temp_dir().join(format!("cell-lab-cli-{}", uuid::Uuid::now_v7()));
    fs::create_dir(&directory).unwrap();
    let checkpoint = directory.join("checkpoint.bin");
    let first = run(&["--seed", "7", "--steps", "5000"], None, &checkpoint);
    assert_eq!(field(&first, "tick"), "5000");
    let left = directory.join("left.bin");
    let right = directory.join("right.bin");
    let continued = run(&["--steps", "100"], Some(&checkpoint), &left);
    let repeated = run(&["--steps", "100"], Some(&checkpoint), &right);
    assert_eq!(field(&continued, "tick"), "5100");
    assert_eq!(field(&first, "session"), field(&continued, "session"));
    for key in ["session", "tick", "cells", "births", "deaths", "energy"] {
        assert_eq!(field(&continued, key), field(&repeated, key));
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn command_line_rejects_invalid_bin_and_preserves_the_existing_save() {
    let directory = std::env::temp_dir().join(format!("cell-lab-invalid-{}", uuid::Uuid::now_v7()));
    fs::create_dir(&directory).unwrap();
    let checkpoint = directory.join("checkpoint.bin");
    let first = run(&["--steps", "100"], None, &checkpoint);
    assert_eq!(field(&first, "tick"), "100");
    let saved = fs::read(&checkpoint).unwrap();
    let mut magic = saved.clone();
    magic[0] = 0;
    let mut version = saved.clone();
    version[8] = 255;
    let invalid = directory.join("invalid.bin");
    for bytes in [
        magic,
        version,
        saved[..20].to_vec(),
        saved[..saved.len() - 1].to_vec(),
        [saved.clone(), vec![0]].concat(),
    ] {
        fs::write(&invalid, bytes).unwrap();
        let output = run(&["--steps", "0"], Some(&invalid), &checkpoint);
        assert!(!output.status.success());
        assert_eq!(fs::read(&checkpoint).unwrap(), saved);
    }
    let definitions = directory.join("changed.ron");
    fs::write(
        &definitions,
        include_str!("../data/definitions.ron")
            .replace("ambient_temperature: 20.0", "ambient_temperature: 21.0"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cell_simulation"))
        .args(["--headless", "--steps", "0", "--definitions"])
        .arg(&definitions)
        .arg("--load")
        .arg(&checkpoint)
        .arg("--save")
        .arg(&checkpoint)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&checkpoint).unwrap(), saved);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn graphic_only_catalog_edits_do_not_invalidate_a_biological_save() {
    let directory = std::env::temp_dir().join(format!("cell-lab-style-{}", uuid::Uuid::now_v7()));
    fs::create_dir(&directory).unwrap();
    let checkpoint = directory.join("checkpoint.bin");
    let first = run(&["--seed", "11", "--steps", "3000"], None, &checkpoint);
    assert_eq!(field(&first, "tick"), "3000");
    let styled = directory.join("styled.ron");
    fs::write(
        &styled,
        include_str!("../data/definitions.ron")
            .replace("CELL LAB / Living systems", "Autre présentation")
            .replace("cell_radius: 0.19", "cell_radius: 0.24")
            .replace("color: (30, 56, 63, 255)", "color: (40, 66, 73, 255)"),
    )
    .unwrap();
    let resumed = Command::new(env!("CARGO_BIN_EXE_cell_simulation"))
        .args(["--headless", "--steps", "50", "--definitions"])
        .arg(&styled)
        .arg("--load")
        .arg(&checkpoint)
        .arg("--save")
        .arg(directory.join("resumed.bin"))
        .output()
        .unwrap();
    let plain = run(
        &["--steps", "50"],
        Some(&checkpoint),
        &directory.join("plain.bin"),
    );
    assert_eq!(field(&resumed, "tick"), "3050");
    for key in ["session", "cells", "births", "deaths", "energy"] {
        assert_eq!(field(&resumed, key), field(&plain, key));
    }
    fs::remove_dir_all(directory).unwrap();
}
