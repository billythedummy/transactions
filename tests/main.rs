use assert_cmd::Command;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

const INPUT_FOLDER: &str = "tests/input";
const OUTPUT_FOLDER: &str = "tests/output";

fn test_csv<P: Clone + AsRef<Path>>(name: P) {
    let mut cmd = Command::cargo_bin("transactions").unwrap();
    let res = cmd
        .arg(
            Path::new(INPUT_FOLDER)
                .join(name.clone())
                .with_extension("csv"),
        )
        .output()
        .unwrap();
    let out = std::str::from_utf8(&res.stdout).unwrap();
    let expected_file = BufReader::new(
        fs::File::open(Path::new(OUTPUT_FOLDER).join(name).with_extension("csv")).unwrap(),
    );
    let mut expected = HashSet::new();
    let mut expected_itr = expected_file.lines();
    expected_itr.next(); // first line is header
    for line in expected_itr {
        expected.insert(line.unwrap());
    }
    // Remove matching lines from set one by one
    let mut out_itr = out.lines();
    out_itr.next(); // first line is header
    for line in out_itr {
        assert!(expected.remove(line));
    }
    assert!(expected.is_empty());
}

#[test]
fn basic() {
    test_csv("basic.csv");
}

#[test]
fn dispute() {
    test_csv("dispute.csv");
}

#[test]
fn multiple_failures() {
    test_csv("multiple_failures.csv");
}
