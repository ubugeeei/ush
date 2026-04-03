use rustyline::{Context, completion::Pair};

use super::{UshHelper, builtin_completion, git_completion, syntax};

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
    _ctx: &Context<'_>,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let prefix = &line[..pos];
    let start = syntax::word_start(line, pos);
    let word = &prefix[start..];
    let trimmed = word.trim();
    let mut pairs = Vec::new();

    if let Some((_offset, needle, brace)) =
        syntax::env_query(word).filter(|(_, needle, _)| !needle.is_empty())
    {
        let suffix = if brace && !word.ends_with('}') {
            "}"
        } else {
            ""
        };
        pairs = helper.env_pairs(&needle, brace, suffix);
    } else if is_export_name_context(prefix, start, word) {
        if !trimmed.is_empty() {
            pairs = helper.env_name_pairs(word);
        }
    } else if syntax::command_position(prefix, start) {
        if !trimmed.is_empty() {
            pairs = helper.command_pairs(word);
        }
    } else if let Some((builtin_start, builtin_pairs)) =
        builtin_completion::complete(helper, line, pos)?
    {
        helper.update_completion(line, pos, builtin_start, &builtin_pairs);
        return Ok((builtin_start, builtin_pairs));
    } else if let Some((git_start, git_pairs)) = git_completion::complete(helper, line, pos)? {
        helper.update_completion(line, pos, git_start, &git_pairs);
        return Ok((git_start, git_pairs));
    } else if wants_path_completion(prefix, start, word) {
        let (path_start, path_pairs) = helper.files.complete_path(line, pos)?;
        helper.update_completion(line, pos, path_start, &path_pairs);
        return Ok((path_start, path_pairs));
    }

    helper.update_completion(line, pos, start, &pairs);
    Ok((start, pairs))
}

fn is_export_name_context(prefix: &str, start: usize, word: &str) -> bool {
    word.find('=').is_none()
        && matches!(
            syntax::previous_token(prefix, start).as_deref(),
            Some("export" | "readonly")
        )
}

fn wants_path_completion(prefix: &str, start: usize, word: &str) -> bool {
    word.starts_with('.')
        || word.starts_with('/')
        || word.starts_with('~')
        || word.contains('/')
        || matches!(
            syntax::previous_token(prefix, start).as_deref(),
            Some(token) if syntax::PATH_COMMANDS.contains(&token)
                || matches!(token, "<" | ">" | ">>" | "<<" )
        )
}

#[cfg(test)]
mod tests {
    use rustyline::{Context, completion::Completer, history::DefaultHistory};

    use crate::repl::UshHelper;

    fn helper() -> UshHelper {
        UshHelper::new(
            vec!["echo".to_string(), "grep".to_string(), "git".to_string()],
            vec!["HOME".to_string(), "PATH".to_string(), "PWD".to_string()],
            std::path::PathBuf::from("."),
        )
    }

    #[test]
    fn completes_commands_after_pipes() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let (_, pairs) = helper()
            .complete("echo hi | gr", 12, &ctx)
            .expect("complete");
        assert!(pairs.iter().any(|pair| pair.replacement == "grep"));
    }

    #[test]
    fn completes_env_expansions_and_exports() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let (_, env_pairs) = helper().complete("echo $PA", 8, &ctx).expect("complete");
        let (_, export_pairs) = helper().complete("export PA", 9, &ctx).expect("complete");

        assert!(env_pairs.iter().any(|pair| pair.replacement == "$PATH"));
        assert!(export_pairs.iter().any(|pair| pair.replacement == "PATH="));
    }

    #[test]
    fn does_not_dump_every_command_on_empty_prompt() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let (_, pairs) = helper().complete("", 0, &ctx).expect("complete");

        assert!(pairs.is_empty());
    }

    #[test]
    fn does_not_dump_every_export_name_when_prefix_is_empty() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let (_, pairs) = helper().complete("export ", 7, &ctx).expect("complete");

        assert!(pairs.is_empty());
    }
}
