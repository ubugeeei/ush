mod cli;
mod runtime_diagnostics;
mod script_docs;
mod test_runner;

use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;

use ush_compiler::{SourceMap, UshCompiler};
use ush_config::UshConfig;
use ush_shell::{SessionStartup, Shell, ShellOptions, run_posix_script};
use ush_tooling::{check_file, format_source};

use crate::cli::{Action, Cli};

enum ScriptMode {
    Ush,
    Posix,
}

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
            Action::Test { targets } => {
                process::exit(test_runner::run(targets, cli.config.as_deref())?)
            }
        }
    }

    let options = ShellOptions::resolve(
        cli.stylish,
        cli.plain,
        cli.no_interaction,
        cli.print_ast,
        &config,
    );
    let startup = session_startup(&cli);

    if let Some(script) = &cli.script {
        let status = match script_mode(script) {
            ScriptMode::Ush => {
                if script_docs::handle_script_doc_request(script, &cli.script_args)? {
                    return Ok(());
                }
                let compiler = UshCompiler;
                let compiled = compiler.compile_file_with_sourcemap(script)?;
                let instrumented =
                    runtime_diagnostics::instrument_compiled_script(script, &compiled);
                let mut shell = Shell::new(config, options)?;
                shell.load_session_startup(&startup)?;
                shell.run_compiled_script(script, &instrumented, &cli.script_args)?
            }
            ScriptMode::Posix => run_posix_script(script, &cli.script_args, &options)?,
        };
        process::exit(status);
    }

    let mut shell = Shell::new(config, options)?;
    shell.load_session_startup(&startup)?;
    if let Some(command) = &cli.command {
        process::exit(shell.execute(command)?);
    }

    shell.run_repl()?;
    Ok(())
}

fn compile_action(input: &Path, output: Option<&Path>, sourcemap: Option<&Path>) -> Result<()> {
    let compiled = UshCompiler.compile_file_with_sourcemap(input)?;
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
    summary: JsonSourceMapSummary,
    sources: Vec<JsonSourceMapSourceLine>,
    lines: Vec<JsonSourceMapLine>,
}

#[derive(Serialize)]
struct JsonSourceMapLine {
    generated_line: usize,
    section: String,
    source_line: Option<usize>,
    generated_text: String,
    source_text: Option<String>,
}

#[derive(Serialize)]
struct JsonSourceMapSummary {
    generated_line_count: usize,
    mapped_line_count: usize,
    unmapped_line_count: usize,
    source_line_count: usize,
    first_mapped_generated_line: Option<usize>,
    last_mapped_generated_line: Option<usize>,
    sections: Vec<JsonSourceMapSectionSummary>,
}

#[derive(Serialize)]
struct JsonSourceMapSectionSummary {
    section: String,
    generated_line_count: usize,
    mapped_line_count: usize,
}

#[derive(Serialize)]
struct JsonSourceMapSourceLine {
    source_line: usize,
    source_text: Option<String>,
    generated_lines: Vec<usize>,
}

fn write_sourcemap_file(
    path: &Path,
    input: &Path,
    output: Option<&Path>,
    sourcemap: &SourceMap,
) -> Result<()> {
    let summary = sourcemap.summary();
    let payload = JsonSourceMap {
        version: 2,
        source: input.display().to_string(),
        generated: output.map(|item| item.display().to_string()),
        summary: JsonSourceMapSummary {
            generated_line_count: summary.generated_line_count,
            mapped_line_count: summary.mapped_line_count,
            unmapped_line_count: summary.unmapped_line_count,
            source_line_count: summary.source_line_count,
            first_mapped_generated_line: summary.first_mapped_generated_line,
            last_mapped_generated_line: summary.last_mapped_generated_line,
            sections: summary
                .sections
                .into_iter()
                .map(|section| JsonSourceMapSectionSummary {
                    section: section.section.label().to_string(),
                    generated_line_count: section.generated_line_count,
                    mapped_line_count: section.mapped_line_count,
                })
                .collect(),
        },
        sources: sourcemap
            .source_index()
            .into_iter()
            .map(|line| JsonSourceMapSourceLine {
                source_line: line.source_line,
                source_text: line.source_text,
                generated_lines: line.generated_lines,
            })
            .collect(),
        lines: sourcemap
            .lines
            .iter()
            .map(|line| JsonSourceMapLine {
                generated_line: line.generated_line,
                section: line.section.label().to_string(),
                source_line: line.source_line,
                generated_text: line.generated_text.clone(),
                source_text: line.source_text.clone(),
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&payload).context("failed to serialize sourcemap")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn script_mode(path: &Path) -> ScriptMode {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ush") => ScriptMode::Ush,
        Some("sh") => ScriptMode::Posix,
        _ => ScriptMode::Posix,
    }
}

fn session_startup(cli: &Cli) -> SessionStartup {
    let is_repl = cli.action.is_none() && cli.script.is_none() && cli.command.is_none();

    SessionStartup {
        load_profile: (cli.login && !cli.no_profile) || cli.profile_file.is_some(),
        load_rc: (is_repl && !cli.no_rc) || cli.rc_file.is_some(),
        profile_file: cli.profile_file.clone(),
        rc_file: cli.rc_file.clone(),
    }
}
