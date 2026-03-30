mod cli;
mod script_docs;

use anyhow::{Context, Result};
use clap::Parser;

use ush_compiler::UshCompiler;
use ush_config::UshConfig;
use ush_shell::{Shell, ShellOptions, run_posix_script};

use crate::cli::{Action, Cli};

fn main() -> Result<()> {
    if script_docs::handle_raw_doc_request()? {
        return Ok(());
    }

    let cli = Cli::parse();
    let config = UshConfig::load(cli.config.as_deref())?;

    if let Some(Action::Compile { input, output }) = &cli.action {
        let compiler = UshCompiler::default();
        let compiled = compiler.compile_file(input)?;

        if let Some(output) = output {
            std::fs::write(output, compiled)
                .with_context(|| format!("failed to write {}", output.display()))?;
        } else {
            print!("{compiled}");
        }
        return Ok(());
    }

    let options = ShellOptions::resolve(
        cli.stylish,
        cli.plain,
        cli.no_interaction,
        cli.print_ast,
        &config,
    );

    if let Some(script) = &cli.script {
        if script.extension().and_then(|ext| ext.to_str()) == Some("ush") {
            if script_docs::handle_script_doc_request(script, &cli.script_args)? {
                return Ok(());
            }
            let compiler = UshCompiler::default();
            let compiled = compiler.compile_file(script)?;
            let mut shell = Shell::new(config, options)?;
            shell.run_compiled_script(script, &compiled, &cli.script_args)?;
            return Ok(());
        }

        run_posix_script(script, &cli.script_args, &options)?;
        return Ok(());
    }

    let mut shell = Shell::new(config, options)?;
    if let Some(command) = &cli.command {
        shell.execute(command)?;
    } else {
        shell.run_repl()?;
    }

    Ok(())
}
