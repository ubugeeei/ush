mod command;
mod core;
mod glob;
mod interactive;
mod interactive_support;
mod introspection;
mod sammary;
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
            "port" => self.handle_port(&args, input),
            "stop" => self.handle_stop(&args, input),
            "jobs" => self.handle_jobs(&args),
            "wait" => self.handle_wait(&args),
            "disown" => self.handle_disown(&args),
            "fg" => self.handle_fg(&args),
            "bg" => self.handle_bg(&args),
            "fsam" | "sammary" => self.handle_sammary(&args),
            "history" => self.handle_history(&args),
            "glob" => self.handle_glob(&args, input),
            "export" => self.handle_export(&args),
            "unset" => self.handle_unset(&args),
            "confirm" => self.handle_confirm(&args, input),
            "input" => self.handle_input(&args, input),
            "select" => self.handle_select(&args, input),
            "env" => self.handle_env(&args, input),
            "command" => self.handle_command_builtin(&args, input),
            "which" => self.handle_lookup(&args, LookupStyle::Path, "which"),
            "type" => self.handle_lookup(&args, LookupStyle::Verbose, "type"),
            "tasks" => self.handle_tasks(&args),
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
        "  port PORT...",
        "  stop [--signal TERM|KILL|INT] [PID ...]",
        "  jobs",
        "  wait [%job]",
        "  disown [%job]",
        "  fg [%job]",
        "  bg [%job]",
        "  sammary [--include-lock] <glob|path>... # recursive file and type summary",
        "  glob <pattern>...      # expand glob patterns, or read them from stdin",
        "  export NAME=value",
        "  unset NAME",
        "  confirm [--default yes|no] [prompt ...]",
        "  input [--default value] [prompt ...]",
        "  select [--prompt text] [--default value] [option ...]",
        "  env [NAME=value] [command ...]",
        "  command -v <name>",
        "  which <name>            # show all matches and mark the current resolution",
        "  type <name>",
        "  tasks [filter ...]      # list discovered make/just/mise/npm/vp tasks",
        "  test EXPR / [ EXPR ]",
        "  history [N]",
        "  source <file>",
        "  rm -rf <path>         # guarded unless --yes or USH_INTERACTION=false",
        "  set USH_KEYMAP=vi     # opt into Vi-style REPL editing, useful in Codex Desktop",
        "",
        "interactive shortcuts:",
        "  Ctrl-A/C/E/L/P/N/U/K/W are always wired for line editing and interrupt flow",
        "  Up/Down history, Option-Up/Down prefix search, Home/End line edge",
        "  Ctrl-Shift-Up/Down and Ctrl-Alt-Shift-Up/Down extend selection to line edges",
        "  Cmd-Left/Right and Cmd-Shift-Left/Right work on terminals that forward Super",
        "",
        "structured helpers:",
        "  len, length, lines, json, xml, html, car, cdr, head, tail, take(...), drop(...), nth(...), enumerate(...), swap, fst, snd, frev, fsort, funiq, fjoin(...), map(...), fmap(...), flat(...), ffmap(...), fzip(...), filter(...), ffilter(...), any(...), fany(...), some(...), fsome(...)",
    ]
    .join("\n")
        + "\n"
}
