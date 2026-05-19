//! Custom panic hook for the `ush` binary.
//!
//! With `panic = "abort"` we never unwind, so the default panic
//! message ends up looking like a raw rustc diagnostic to anyone
//! whose shell just died. Replacing the hook lets us print a
//! consistent, end-user-friendly message that:
//!
//! 1. clearly identifies which binary panicked,
//! 2. surfaces the panic payload + source location, and
//! 3. points at the bug tracker with the version string already
//!    in the line, so a copy-paste-into-issue actually contains
//!    enough information to be triaged.

use std::panic::PanicHookInfo;

/// Install the custom panic hook. Should be called once, from
/// `main()`, before any other work runs.
pub fn install() {
    std::panic::set_hook(Box::new(format_panic));
}

fn format_panic(info: &PanicHookInfo<'_>) {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<no message>");

    let location = info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "<unknown location>".to_string());

    eprintln!("ush: internal error: {message}");
    eprintln!("        at {location}");
    eprintln!("        version {}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("This is a bug in ush. Please report it at");
    eprintln!("    https://github.com/ubugeeei/ush/issues/new");
    eprintln!("and include the message above plus a small reproduction.");
}
