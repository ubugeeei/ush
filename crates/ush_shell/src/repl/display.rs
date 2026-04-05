use rustyline::completion::Pair;

pub(crate) const DETAIL_SEPARATOR: &str = "  :  ";

pub(crate) fn same_pair(value: String, detail: Option<&str>) -> Pair {
    let display = render(value.as_str(), detail);
    Pair {
        display,
        replacement: value,
    }
}

pub(crate) fn pair(display: &str, replacement: String, detail: Option<&str>) -> Pair {
    Pair {
        display: render(display, detail),
        replacement,
    }
}

pub(crate) fn render(name: &str, detail: Option<&str>) -> String {
    match detail {
        Some(detail) => {
            let mut display =
                String::with_capacity(name.len() + DETAIL_SEPARATOR.len() + detail.len());
            display.push_str(name);
            display.push_str(DETAIL_SEPARATOR);
            display.push_str(detail);
            display
        }
        None => name.to_string(),
    }
}

pub(crate) fn split(candidate: &str) -> (&str, Option<&str>) {
    if let Some(parts) = split_once(candidate, DETAIL_SEPARATOR) {
        return parts;
    }
    if let Some(parts) = split_once(candidate, "\t") {
        return parts;
    }
    split_once(candidate, "  ").unwrap_or((candidate, None))
}

fn split_once<'a>(candidate: &'a str, separator: &str) -> Option<(&'a str, Option<&'a str>)> {
    candidate.find(separator).map(|index| {
        (
            &candidate[..index],
            Some(&candidate[index + separator.len()..]),
        )
    })
}
