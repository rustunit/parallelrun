use std::process::Command;

#[test]
fn test_simple() {
    let output = Command::new("target/debug/parallelrun")
        .args(vec!["echo 1 && sleep 1", "sleep 2 && echo 2"])
        .output()
        .unwrap();

    let expected_output = r"[0] 1
[0] echo 1 && sleep 1 exited with code 0
[1] 2
[1] sleep 2 && echo 2 exited with code 0
";
    assert_eq!(&String::from_utf8_lossy(&output.stdout), expected_output);
}

#[test]
fn test_kill() {
    let output = Command::new("target/debug/parallelrun")
        .args(vec!["echo 1 && sleep 2", "sleep 1 && echo 2 && sleep 2"])
        .output()
        .unwrap();

    let expected_output = r"[0] 1
[1] 2
[0] echo 1 && sleep 2 exited with code 0
[1] sleep 1 && echo 2 && sleep 2 exited with code 0
";
    assert_eq!(&String::from_utf8_lossy(&output.stdout), expected_output);
}
