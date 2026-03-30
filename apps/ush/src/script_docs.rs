use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, bail};
use ush_compiler::UshCompiler;

pub fn handle_raw_doc_request() -> Result<bool> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let Some((script, request)) = parse_raw_request(&args) else {
        return Ok(false);
    };
    render_doc_request(&script, request)?;
    Ok(true)
}

pub fn handle_script_doc_request(script: &Path, args: &[String]) -> Result<bool> {
    let Some(request) = parse_request(args) else {
        return Ok(false);
    };
    render_doc_request(script, request)?;
    Ok(true)
}

#[derive(Clone)]
enum DocRequest {
    Help,
    Man(Option<String>),
    Complete(Option<String>),
}

fn parse_raw_request(args: &[OsString]) -> Option<(PathBuf, DocRequest)> {
    let script = PathBuf::from(args.first()?.clone());
    script
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| *ext == "ush")?;
    let request = parse_request(
        &args[1..]
            .iter()
            .filter_map(|arg| arg.to_str().map(str::to_string))
            .collect::<Vec<_>>(),
    )?;
    Some((script, request))
}

fn parse_request(args: &[String]) -> Option<DocRequest> {
    match args.first()?.as_str() {
        "-h" | "--help" => Some(DocRequest::Help),
        "man" | "--man" => Some(DocRequest::Man(args.get(1).cloned())),
        "complete" | "--complete" => Some(DocRequest::Complete(args.get(1).cloned())),
        _ => None,
    }
}

fn render_doc_request(script: &Path, request: DocRequest) -> Result<()> {
    let compiler = UshCompiler::default();
    let compiled = compiler.compile_file(script)?;
    let temp = env::temp_dir().join(format!("ush-doc-{}.sh", std::process::id()));
    fs::write(&temp, compiled)?;
    let mut command = Command::new("/bin/sh");
    command.arg(&temp);
    match request {
        DocRequest::Help => {
            command.arg("--help");
        }
        DocRequest::Man(item) => {
            command.arg("--man");
            if let Some(item) = item {
                command.arg(item);
            }
        }
        DocRequest::Complete(prefix) => {
            command.arg("--complete");
            if let Some(prefix) = prefix {
                command.arg(prefix);
            }
        }
    }
    let output = command.output()?;
    let _ = fs::remove_file(&temp);
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}
