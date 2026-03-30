use rustyline::{Context, completion::Pair};

use super::{UshHelper, syntax};

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
    _ctx: &Context<'_>,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let prefix = &line[..pos];
    let start = syntax::word_start(line, pos);
    let word = &prefix[start..];

    if let Some((_offset, needle, brace)) = syntax::env_query(word) {
        let suffix = if brace && !word.ends_with('}') {
            "}"
        } else {
            ""
        };
        return Ok((start, helper.env_pairs(&needle, brace, suffix)));
    }

    if is_export_name_context(prefix, start, word) {
        return Ok((start, helper.env_name_pairs(word)));
    }

    if syntax::command_position(prefix, start) {
        return Ok((start, helper.command_pairs(word)));
    }

    if wants_path_completion(prefix, start, word) {
        return helper.files.complete_path(line, pos);
    }

    Ok((start, Vec::new()))
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
}
