use crate::parser::{Child, Element};
use std::{borrow::Cow, iter::once};

const QUOTE_CHARS: [char; 3] = ['"', '\'', '`'];

struct TermInfo {
    must_quote: bool,
    has_double_quote: bool,
    has_single_quote: bool,
    has_backtick_quote: bool,
}

fn gather_term_info(text: &str) -> TermInfo {
    // conditions for needing to quote a term:
    // - it starts with a quote char, or
    // - it contains a space

    // if it starts with any quote char, then it will always be quoted
    let mut must_quote = text.starts_with(&QUOTE_CHARS);
    let mut has_double_quote = false;
    let mut has_single_quote = false;
    let mut has_backtick_quote = false;

    for x in text.chars() {
        if x == ' ' {
            must_quote = true;
        } else if x == '"' {
            has_double_quote = true;
        } else if x == '\'' {
            has_single_quote = true;
        } else if x == '`' {
            has_backtick_quote = true;
        }
    }

    TermInfo {
        must_quote,
        has_double_quote,
        has_single_quote,
        has_backtick_quote,
    }
}

fn serialise_term<'a>(text: &'a str) -> Cow<'a, str> {
    if text.is_empty() {
        return "\"\"".into();
    }

    let info = gather_term_info(text);

    if info.must_quote {
        let mut replace_backticks = false;
        let quote_char = if !info.has_double_quote {
            '"'
        } else if !info.has_single_quote {
            '\''
        } else if !info.has_backtick_quote {
            '`'
        } else {
            // Reaper does not allow terms with all 3 quote characters.
            // Backticks will be replaced with single quotes.
            replace_backticks = true;
            '`'
        };

        let quoted_text: String = if replace_backticks {
            once(quote_char)
                .chain(text.chars().map(|x| if x == '`' { '\'' } else { x }))
                .chain(once(quote_char))
                .collect()
        } else {
            once(quote_char)
                .chain(text.chars())
                .chain(once(quote_char))
                .collect()
        };

        quoted_text.into()
    } else {
        text.into()
    }
}

/// Serialise an element back to a [String] following the RPP format.
pub fn serialize_to_string(element: &Element) -> String {
    let mut buf = String::new();
    process(&mut buf, element, 0);
    buf
}

fn process(buf: &mut String, element: &Element, indent_level: usize) {
    // first line
    for _ in 0..indent_level {
        buf.push_str("  ")
    }
    buf.push('<');
    buf.push_str(element.tag);

    for x in &element.attr {
        let x = serialise_term(x);
        buf.push(' ');
        buf.push_str(&x);
    }

    buf.push('\n');

    for child in element.children.iter() {
        match child {
            Child::Line(child) => {
                for _ in 0..(indent_level + 1) {
                    buf.push_str("  ")
                }

                let mut is_first = true;
                for x in child {
                    let x = serialise_term(x);
                    if is_first {
                        is_first = false;
                    } else {
                        buf.push(' ')
                    }
                    buf.push_str(&x);
                }

                buf.push('\n');
            }
            Child::Element(child) => {
                process(buf, child, indent_level + 1);
                buf.push('\n');
            }
        }
    }

    // last line
    for _ in 0..indent_level {
        buf.push_str("  ")
    }
    buf.push('>');
}

#[cfg(test)]
mod serialise_term_tests {
    use super::*;

    #[test]
    fn test_01() {
        let input = r#"as d"#;
        let expected = r#""as d""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_02() {
        let input = r#"'as d'"#;
        let expected = r#""'as d'""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_03() {
        let input = r#""as d""#;
        let expected = r#"'"as d"'"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_04() {
        let input = r#"`as d`"#;
        let expected = r#""`as d`""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_05() {
        let input = r#"'as d"as d'"#;
        let expected = r#"`'as d"as d'`"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_06() {
        let input = r#""as d`as d""#;
        let expected = r#"'"as d`as d"'"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_07() {
        let input = r#"`as d'as d`"#;
        let expected = r#""`as d'as d`""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_08() {
        let input = r#"a'"`b"#;
        let expected = r#"a'"`b"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_09() {
        let input = r#"a'"`b       a'"`b"#;
        let expected = r#"`a'"'b       a'"'b`"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_10() {
        let input = r#"'asd"#;
        let expected = r#""'asd""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_11() {
        let input = r#""asd"#;
        let expected = r#"'"asd'"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_12() {
        let input = r#"`asd"#;
        let expected = r#""`asd""#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_13() {
        let input = r#"asd'"#;
        let expected = r#"asd'"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_14() {
        let input = r#"asd`"#;
        let expected = r#"asd`"#;
        assert_eq!(serialise_term(input), expected);
    }

    #[test]
    fn test_15() {
        let input = r#"asd""#;
        let expected = r#"asd""#;
        assert_eq!(serialise_term(input), expected);
    }
}
