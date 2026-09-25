use super::*;

/// Renders a layout with the cursor in brackets.
fn render(value: &str, cursor: usize, width: u16) -> String {
    let l = layout(value, cursor, width);
    format!("{}[{}]{}{}", l.before, l.cursor, l.after, l.padding())
}

#[test]
fn ascii() {
    assert_eq!(render("hello", 5, 4), "llo[ ]");
    assert_eq!(render("hello", 1, 4), "[e]llo");
    assert_eq!(render("hello", 0, 4), "[h]ell");
    assert_eq!(render("hi", 1, 5), "h[i]   ");
}

#[test]
fn wide_before_cursor() {
    assert_eq!(render("a👍b", 3, 4), "👍b[ ]");
    // Half of 👍 would fit, so a column is left blank instead.
    assert_eq!(render("a👍b", 3, 3), "b[ ] ");
}

#[test]
fn wide_under_cursor() {
    let heart = "❤\u{FE0F}\u{200D}🔥";
    assert_eq!(render(&format!("a{heart}b"), 1, 5), format!("a[{heart}]b "));
    assert_eq!(render(&format!("a{heart}b"), 1, 4), format!("[{heart}]b "));
}

#[test]
fn wide_after_cursor() {
    let eye = "👁\u{FE0F}\u{200D}🗨\u{FE0F}";
    assert_eq!(render(&format!("ab{eye}"), 0, 3), "[a]b ");
    assert_eq!(render(&format!("ab{eye}"), 0, 4), format!("[a]b{eye}"));
}
