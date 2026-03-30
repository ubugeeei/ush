mod command;
mod core;
mod introspection;
mod test_eval;

use anyhow::{Result, bail};

use self::{core::render_echo, introspection::LookupStyle};
use super::{Shell, ValueStream, parser::CommandSpec};

impl Shell {
    pub(crate) fn execute_builtin(
        &mut self,
        spec: &CommandSpec,
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let args = self.expand_args(&spec.args)?;

        match spec.command.as_str() {
            ":" => Ok((ValueStream::Empty, 0)),
            "." => self.handle_source(&args),
            "cd" => self.change_directory(&args),
            "pwd" => Ok((ValueStream::Text(self.render_pwd()), 0)),
            "echo" => Ok((ValueStream::Text(render_echo(&args)), 0)),
            "true" => Ok((ValueStream::Empty, 0)),
            "false" => Ok((ValueStream::Empty, 1)),
            "alias" => self.handle_alias(&args),
            "unalias" => self.handle_unalias(&args),
            "history" => Ok((ValueStream::Text(self.read_history()), 0)),
            "export" => self.handle_export(&args),
            "unset" => self.handle_unset(&args),
            "env" => self.handle_env(&args, input),
            "command" => self.handle_command_builtin(&args, input),
            "which" => self.handle_lookup(&args, LookupStyle::Path, "which"),
            "type" => self.handle_lookup(&args, LookupStyle::Verbose, "type"),
            "test" | "[" => self.handle_test(&spec.command, &args),
            "help" => Ok((ValueStream::Text(help_text()), 0)),
            "source" => self.handle_source(&args),
            "exit" => self.handle_exit(&args),
            "rm" => self.execute_rm(&args, input),
            other => bail!("unsupported builtin: {other}"),
        }
    }
}

fn help_text() -> String {
    [
        "ush builtins:",
        "  :                     # no-op",
        "  . <file>              # alias for source",
        "  cd <dir>",
        "  pwd",
        "  echo [-n] ...",
        "  true / false",
        "  alias name=value",
        "  unalias name",
        "  export NAME=value",
        "  unset NAME",
        "  env [NAME=value] [command ...]",
        "  command -v <name>",
        "  which <name>",
        "  type <name>",
        "  test EXPR / [ EXPR ]",
        "  history",
        "  source <file>",
        "  rm -rf <path>         # guarded unless --yes or USH_INTERACTION=false",
        "",
        "interactive shortcuts:",
        "  Up/Down history, Option-Up/Down prefix search, Home/End line edge",
        "",
        "structured helpers:",
        "  len, length, lines, json, xml, html, map(...), filter(...), any(...)",
    ]
    .join("\n")
        + "\n"
}
