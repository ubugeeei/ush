mod cli;
mod script_docs;

use std::process;

use anyhow::{Context, Result};
use clap::Parser;

use ush_compiler::UshCompiler;
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
            Action::Compile { input, output } => {
                compile_action(input, output.as_deref())?;
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

fn compile_action(input: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    let compiled = UshCompiler::default().compile_file(input)?;
    if let Some(output) = output {
        std::fs::write(output, compiled)
            .with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        print!("{compiled}");
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
