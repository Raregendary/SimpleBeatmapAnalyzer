use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn sba() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sba"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sba-cli-{}-{}-{}",
        std::process::id(),
        name,
        unique
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn cli_prints_csv_for_a_single_file() {
    let output = sba()
        .arg(fixtures().join("standard_50.osu"))
        .args(["--columns", "Title,Stars,MapSetID"])
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let text = stdout(&output);
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("Title,Stars,MapSetID"));
    assert!(lines
        .next()
        .unwrap()
        .starts_with("Fixture Song: The Rebirth,3.95,6789"));
    assert!(stderr(&output).contains("1 computed"));
}

#[test]
fn cli_writes_selected_columns_to_a_file() {
    let dir = temp_dir("output");
    let target = dir.join("out.csv");

    let output = sba()
        .arg(fixtures().join("standard_50.osu"))
        .args(["--columns", "Title,Streams"])
        .arg("--output")
        .arg(&target)
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(target.is_file(), "output file was not created");

    let text = std::fs::read_to_string(&target).unwrap();
    assert!(text.starts_with("Title,Streams\n"));
    assert!(text.contains("Fixture Song: The Rebirth,40.00"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_emits_json_when_requested() {
    let output = sba()
        .arg(fixtures().join("standard_50.osu"))
        .args(["--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let text = stdout(&output);
    assert!(text.trim_start().starts_with('['));
    assert!(text.contains("\"title\": \"Fixture Song: The Rebirth\""));
    assert!(text.contains("\"stream_notes\": 20"));
    assert!(text.contains("\"beatmap_set_id\": 6789"));
}

#[test]
fn cli_cache_reuses_analyses() {
    let dir = temp_dir("cache");
    let cache = dir.join("cache.bin");
    let output_csv = dir.join("out.csv");

    let first = sba()
        .arg(fixtures())
        .args(["--all-modes", "--no-progress"])
        .arg("--output")
        .arg(&output_csv)
        .arg("--cache")
        .arg(&cache)
        .output()
        .unwrap();
    assert!(first.status.success(), "stderr: {}", stderr(&first));
    assert!(stderr(&first).contains("5 computed"));
    assert!(cache.is_file());

    let second = sba()
        .arg(fixtures())
        .args(["--all-modes", "--no-progress"])
        .arg("--output")
        .arg(&output_csv)
        .arg("--cache")
        .arg(&cache)
        .output()
        .unwrap();
    assert!(second.status.success(), "stderr: {}", stderr(&second));
    assert!(
        stderr(&second).contains("0 computed, 5 cached"),
        "stderr: {}",
        stderr(&second)
    );

    let rows = std::fs::read_to_string(&output_csv)
        .unwrap()
        .lines()
        .count();
    assert_eq!(rows, 6, "header plus five fixtures");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_cache_is_option_aware() {
    let dir = temp_dir("cache_options");
    let cache = dir.join("cache.bin");
    let output_csv = dir.join("out.csv");

    let first = sba()
        .arg(fixtures())
        .args(["--all-modes", "--no-progress"])
        .arg("--output")
        .arg(&output_csv)
        .arg("--cache")
        .arg(&cache)
        .output()
        .unwrap();
    assert!(first.status.success(), "stderr: {}", stderr(&first));

    // A different threshold must not reuse the previous results.
    let changed = sba()
        .arg(fixtures())
        .args(["--all-modes", "--no-progress", "--stream-distance", "100"])
        .arg("--output")
        .arg(&output_csv)
        .arg("--cache")
        .arg(&cache)
        .output()
        .unwrap();
    assert!(changed.status.success(), "stderr: {}", stderr(&changed));
    assert!(
        stderr(&changed).contains("5 computed, 0 cached"),
        "stderr: {}",
        stderr(&changed)
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cli_rejects_unknown_options_and_bad_values() {
    let unknown = sba()
        .arg("--nope")
        .arg(fixtures().join("standard_50.osu"))
        .output()
        .unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(stderr(&unknown).contains("unknown option"));

    let bad_accuracy = sba()
        .arg(fixtures().join("standard_50.osu"))
        .args(["--accuracy", "150"])
        .output()
        .unwrap();
    assert_eq!(bad_accuracy.status.code(), Some(2));
    assert!(stderr(&bad_accuracy).contains("accuracy"));
}

#[test]
fn cli_lists_columns() {
    let output = sba().arg("--list-columns").output().unwrap();

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Streams"));
    assert!(text.contains("DeathStreams"));
    assert!(text.contains("AvgJumpsDistance"));
}
