use std::process::Command;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn car_and_cdr_work_as_pipeline_helpers() {
    let car = ush()
        .args(["-c", "printf 'a\nb\nc\n' | car"])
        .output()
        .expect("run ush");
    let cdr = ush()
        .args(["-c", "printf 'a\nb\nc\n' | cdr"])
        .output()
        .expect("run ush");

    assert!(car.status.success());
    assert!(cdr.status.success());
    assert_eq!(String::from_utf8_lossy(&car.stdout), "a\n");
    assert_eq!(String::from_utf8_lossy(&cdr.stdout), "b\nc\n");
}

#[test]
fn flat_helper_can_rebuild_and_extend_a_stream() {
    let output = ush()
        .args([
            "-c",
            r#"printf 'a\nb\nc\n' | flat(\head, rest -> [head, "tail", rest])"#,
        ])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\ntail\nb\nc\n");
}

#[test]
fn fmap_ffmap_and_fzip_work_as_functional_aliases() {
    let fmap = ush()
        .args(["-c", r#"printf 'ush\n' | fmap(\it -> upper(it))"#])
        .output()
        .expect("run ush");
    let ffmap = ush()
        .args([
            "-c",
            r#"printf 'a\nb\nc\n' | ffmap(\head, rest -> [head, rest])"#,
        ])
        .output()
        .expect("run ush");
    let fzip = ush()
        .args(["-c", r#"printf 'a\nb\n' | fzip(["1", "2"])"#])
        .output()
        .expect("run ush");

    assert!(fmap.status.success());
    assert!(ffmap.status.success());
    assert!(fzip.status.success());
    assert_eq!(String::from_utf8_lossy(&fmap.stdout), "USH\n");
    assert_eq!(String::from_utf8_lossy(&ffmap.stdout), "a\nb\nc\n");
    assert_eq!(String::from_utf8_lossy(&fzip.stdout), "a\t1\nb\t2\n");
}
