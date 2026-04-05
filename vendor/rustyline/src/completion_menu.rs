use std::borrow::Cow;

use crate::completion::Candidate;
use crate::config::CompletionType;
use crate::highlight::Highlighter;
use crate::layout::Unit;

pub fn render<C: Candidate>(
    candidates: &[C],
    active: usize,
    rows: Unit,
    highlighter: Option<&dyn Highlighter>,
) -> String {
    let max_items = visible_limit(rows);
    let (start, end) = visible_range(candidates.len(), active, max_items);
    let mut menu = String::with_capacity(candidates.len().min(max_items) * 24 + 64);

    push_status(&mut menu, active + 1, candidates.len());
    if start > 0 {
        menu.push('\n');
        menu.push_str("  ...");
    }
    for index in start..end {
        menu.push('\n');
        if index == active {
            menu.push('>');
        } else {
            menu.push(' ');
        }
        menu.push(' ');
        push_candidate(&mut menu, &candidates[index], index == active, highlighter);
    }
    if end < candidates.len() {
        menu.push('\n');
        menu.push_str("  ...");
    }

    menu
}

fn visible_limit(rows: Unit) -> usize {
    let rows = usize::from(rows.saturating_sub(4));
    rows.clamp(1, 10)
}

fn visible_range(total: usize, active: usize, max_items: usize) -> (usize, usize) {
    if total <= max_items {
        return (0, total);
    }
    let half = max_items / 2;
    let mut start = active.saturating_sub(half);
    let mut end = start + max_items;
    if end > total {
        end = total;
        start = end - max_items;
    }
    (start, end)
}

fn push_status(buf: &mut String, active: usize, total: usize) {
    buf.push('[');
    push_usize(buf, active);
    buf.push('/');
    push_usize(buf, total);
    buf.push_str("] tab next  shift-tab prev  enter accept  esc cancel");
}

fn push_candidate<C: Candidate>(
    buf: &mut String,
    candidate: &C,
    active: bool,
    highlighter: Option<&dyn Highlighter>,
) {
    let display = candidate.display();
    match highlighter {
        Some(value) => {
            let highlighted =
                value.highlight_candidate_with_state(display, CompletionType::Circular, active);
            match highlighted {
                Cow::Borrowed(text) => buf.push_str(text),
                Cow::Owned(text) => buf.push_str(text.as_str()),
            }
        }
        None => buf.push_str(display),
    }
}

fn push_usize(buf: &mut String, mut value: usize) {
    let mut digits = [0_u8; 20];
    let mut len = 1;

    digits[0] = (value % 10) as u8;
    value /= 10;
    while value > 0 {
        digits[len] = (value % 10) as u8;
        value /= 10;
        len += 1;
    }
    while len > 0 {
        len -= 1;
        buf.push(char::from(b'0' + digits[len]));
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::completion::Pair;
    use crate::highlight::Highlighter;

    struct TestHighlighter;

    impl Highlighter for TestHighlighter {
        fn highlight_candidate_with_state<'c>(
            &self,
            candidate: &'c str,
            _: crate::CompletionType,
            active: bool,
        ) -> Cow<'c, str> {
            if !active {
                return Cow::Borrowed(candidate);
            }
            let mut styled = String::from("[");
            styled.push_str(candidate);
            styled.push(']');
            Cow::Owned(styled)
        }
    }

    #[test]
    fn renders_selected_candidate_and_status() {
        let candidates = [
            pair("alpha"),
            pair("beta"),
            pair("gamma"),
            pair("delta"),
            pair("epsilon"),
        ];
        let menu = super::render(&candidates, 2, 8, Some(&TestHighlighter));

        assert!(menu.starts_with("[3/5]"));
        assert!(menu.contains("\n> [gamma]"));
        assert!(menu.contains("\n  beta"));
    }

    #[test]
    fn trims_large_lists_around_active_candidate() {
        let candidates = [
            pair("a"),
            pair("b"),
            pair("c"),
            pair("d"),
            pair("e"),
            pair("f"),
            pair("g"),
            pair("h"),
            pair("i"),
        ];
        let menu = super::render(&candidates, 7, 7, None);

        assert!(menu.contains("\n  ..."));
        assert!(menu.contains("\n> h"));
    }

    fn pair(value: &str) -> Pair {
        Pair {
            display: value.to_string(),
            replacement: value.to_string(),
        }
    }
}
