pub mod dialog;
pub mod monitor;
pub mod window;

// Takes a `&'static str` here since we convert clickable hyperlinks.
// DO NOT pass in untrusted input.
#[expect(
    dead_code,
    reason = "Public Windows error helper retained for callers that need dialog reporting"
)]
pub fn error(err: &'static str) {
    dialog::error(err);
}
