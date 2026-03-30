mod cli;
mod script_docs;

use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;

use ush_compiler::{SourceMap, UshCompiler};
use ush_config::UshConfig;
use ush_shell::{Shell, ShellOptions, run_posix_script};
use ush_tooling::{check_file, format_source};

use crate::cli::{Action, Cli};

fn main() -> Result<()> {
    if script_docs::handle_raw_doc_request()? {
        return Ok(());
    }

    let cli = Cli::parse();
    let config = UshConfig::load(cli.config.as_deref())?;

    if let Some(action) = &cli.action {
        match action {
            Action::Compile {
                input,
                output,
                sourcemap,
            } => {
                compile_action(input, output.as_deref(), sourcemap.as_deref())?;
                return Ok(());
            }
            Action::Format {
                input,
                check,
                stdout,
            } => process::exit(format_action(input, *check, *stdout)?),
            Action::Check { input } => process::exit(check_action(input)?),
        }
    }

    let options = ShellOptions::resolve(
        cli.stylish,
        cli.plain,
        cli.no_interaction,
        cli.print_ast,
        &config,
    );

    if let Some(script) = &cli.script {
        let status = if script.extension().and_then(|ext| ext.to_str()) == Some("ush") {
            if script_docs::handle_script_doc_request(script, &cli.script_args)? {
                return Ok(());
            }
            let compiler = UshCompiler::default();
            let compiled = compiler.compile_file(script)?;
            let mut shell = Shell::new(config, options)?;
            shell.run_compiled_script(script, &compiled, &cli.script_args)?
        } else {
            run_posix_script(script, &cli.script_args, &options)?
        };
        process::exit(status);
    }

    let mut shell = Shell::new(config, options)?;
    if let Some(command) = &cli.command {
        process::exit(shell.execute(command)?);
    }

    shell.run_repl()?;
    Ok(())
}

fn compile_action(input: &Path, output: Option<&Path>, sourcemap: Option<&Path>) -> Result<()> {
    let compiled = UshCompiler::default().compile_file_with_sourcemap(input)?;
    if let Some(output) = output {
        std::fs::write(output, &compiled.shell)
            .with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        print!("{}", compiled.shell);
    }
    if let Some(sourcemap) = sourcemap {
        write_sourcemap_file(sourcemap, input, output, &compiled.sourcemap)?;
    }
    Ok(())
}

fn format_action(input: &std::path::Path, check: bool, stdout: bool) -> Result<i32> {
    let source = std::fs::read_to_string(input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let formatted = format_source(&source);
    if check {
        if source == formatted {
            return Ok(0);
        }
        eprintln!("ush: formatting required: {}", input.display());
        return Ok(1);
    }
    if stdout {
        print!("{formatted}");
        return Ok(0);
    }
    std::fs::write(input, formatted)
        .with_context(|| format!("failed to write {}", input.display()))?;
    Ok(0)
}

fn check_action(input: &std::path::Path) -> Result<i32> {
    let diagnostics = check_file(input)?;
    if diagnostics.is_empty() {
        return Ok(0);
    }
    for diagnostic in diagnostics {
        eprintln!(
            "{}:{}: {}",
            input.display(),
            diagnostic.line + 1,
            diagnostic.message
        );
    }
    Ok(1)
}

#[derive(Serialize)]
struct JsonSourceMap {
    version: u32,
    source: String,
    generated: Option<String>,
    lines: Vec<JsonSourceMapLine>,
}

#[derive(Serialize)]
struct JsonSourceMapLine {
    generated_line: usize,
    source_line: Option<usize>,
}

fn write_sourcemap_file(
    path: &Path,
    input: &Path,
    output: Option<&Path>,
    sourcemap: &SourceMap,
) -> Result<()> {
    let payload = JsonSourceMap {
        version: 1,
        source: input.display().to_string(),
        generated: output.map(|item| item.display().to_string()),
        lines: sourcemap
            .lines
            .iter()
            .map(|line| JsonSourceMapLine {
                generated_line: line.generated_line,
                source_line: line.source_line,
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&payload).context("failed to serialize sourcemap")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}
