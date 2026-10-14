//! Check that the foundation binary cannot report an unimplemented solve as success.

use std::process::Command;

#[test]
fn unsupported_solve_fails_without_success_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrasim"))
        .args(["solve", "missing.json"])
        .output()
        .expect("the built CLI must be executable");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented"));
}
