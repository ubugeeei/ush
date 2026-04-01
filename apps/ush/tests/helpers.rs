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
fn head_tail_take_drop_and_nth_work_on_line_streams() {
    let head = ush()
        .args(["-c", "printf 'a\nb\nc\n' | head"])
        .output()
        .expect("run ush");
    let tail = ush()
        .args(["-c", "printf 'a\nb\nc\n' | tail"])
        .output()
        .expect("run ush");
    let take = ush()
        .args(["-c", "printf 'a\nb\nc\n' | take(2)"])
        .output()
        .expect("run ush");
    let drop = ush()
        .args(["-c", "printf 'a\nb\nc\n' | drop(1)"])
        .output()
        .expect("run ush");
    let nth = ush()
        .args(["-c", "printf 'a\nb\nc\n' | nth(1)"])
        .output()
        .expect("run ush");
    let enumerate = ush()
        .args(["-c", "printf 'a\nb\nc\n' | enumerate(1)"])
        .output()
        .expect("run ush");

    assert_eq!(String::from_utf8_lossy(&head.stdout), "a\n");
    assert_eq!(String::from_utf8_lossy(&tail.stdout), "b\nc\n");
    assert_eq!(String::from_utf8_lossy(&take.stdout), "a\nb\n");
    assert_eq!(String::from_utf8_lossy(&drop.stdout), "b\nc\n");
    assert_eq!(String::from_utf8_lossy(&nth.stdout), "b\n");
    assert_eq!(
        String::from_utf8_lossy(&enumerate.stdout),
        "1\ta\n2\tb\n3\tc\n"
    );
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
    let fst = ush()
        .args(["-c", r#"printf 'a\nb\n' | fzip(["1", "2"]) | fst"#])
        .output()
        .expect("run ush");
    let snd = ush()
        .args(["-c", r#"printf 'a\nb\n' | fzip(["1", "2"]) | snd"#])
        .output()
        .expect("run ush");
    let swap = ush()
        .args(["-c", r#"printf 'a\nb\n' | fzip(["1", "2"]) | swap"#])
        .output()
        .expect("run ush");
    let ffilter = ush()
        .args([
            "-c",
            r#"printf 'ush\nmini\n' | ffilter(\it -> contains(it, "u"))"#,
        ])
        .output()
        .expect("run ush");
    let fany = ush()
        .args([
            "-c",
            r#"printf 'ush\nmini\n' | fany(\it -> contains(it, "u"))"#,
        ])
        .output()
        .expect("run ush");

    assert!(fmap.status.success());
    assert!(ffmap.status.success());
    assert!(fzip.status.success());
    assert!(ffilter.status.success());
    assert!(fany.status.success());
    assert_eq!(String::from_utf8_lossy(&fmap.stdout), "USH\n");
    assert_eq!(String::from_utf8_lossy(&ffmap.stdout), "a\nb\nc\n");
    assert_eq!(String::from_utf8_lossy(&fzip.stdout), "a\t1\nb\t2\n");
    assert_eq!(String::from_utf8_lossy(&fst.stdout), "a\nb\n");
    assert_eq!(String::from_utf8_lossy(&snd.stdout), "1\n2\n");
    assert_eq!(String::from_utf8_lossy(&swap.stdout), "1\ta\n2\tb\n");
    assert_eq!(String::from_utf8_lossy(&ffilter.stdout), "ush\n");
    assert_eq!(String::from_utf8_lossy(&fany.stdout), "true\n");
}
