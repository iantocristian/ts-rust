use crate::emit_text_writer::decode_last_rune;
use crate::{get_default_indent_size, EmitTextWriter, SingleLineStringWriter, TextWriter};

#[test]
fn text_writer_indents_at_line_start_and_tracks_lines() {
    let mut w = TextWriter::new(b"\n", 0);
    assert_eq!(get_default_indent_size(), 4);
    assert!(w.is_at_start_of_line());
    w.increase_indent();
    assert_eq!(w.get_column(), 4, "pending indent counts before any text");
    w.write(b"a");
    assert_eq!(w.text(), b"    a");
    assert_eq!((w.get_line(), w.get_column(), w.get_indent()), (0, 5, 1));
    w.write_line();
    w.write_line();
    assert_eq!(
        w.text(),
        b"    a\n",
        "a second WriteLine at line start writes nothing"
    );
    assert_eq!(w.get_line(), 1);
    w.write_line_force(true);
    assert_eq!(w.text(), b"    a\n\n");
    w.decrease_indent();
    w.write(b"b");
    assert_eq!(w.text(), b"    a\n\nb");
    assert_eq!(w.get_text_pos(), 8);
}

#[test]
fn text_writer_counts_embedded_line_breaks_and_utf16_columns() {
    let mut w = TextWriter::new(b"\r\n", 2);
    w.write("x\r\ny\u{1F600}".as_bytes());
    assert_eq!(w.get_line(), 1);
    assert_eq!(
        w.get_column(),
        3,
        "y plus an astral scalar is three UTF-16 units"
    );
    assert!(!w.is_at_start_of_line());
    w.raw_write(b"z\n");
    assert_eq!(w.get_line(), 2);
    assert!(
        w.is_at_start_of_line(),
        "text ending in a break leaves the writer at line start"
    );
    // Upstream updates line state even for an empty RawWrite.
    w.raw_write(b"");
    assert!(!w.is_at_start_of_line());
    w.write_line();
    assert_eq!(w.text(), "x\r\ny\u{1F600}z\n\r\n".as_bytes());
}

#[test]
fn text_writer_trailing_state_and_clear() {
    let mut w = TextWriter::new(b"\n", 4);
    assert!(!w.has_trailing_whitespace());
    w.write_comment(b"// c");
    assert!(w.has_trailing_comment());
    w.write(b"");
    assert!(
        w.has_trailing_comment(),
        "an empty write does not clear the comment state"
    );
    w.write(b"a ");
    assert!(!w.has_trailing_comment());
    assert!(w.has_trailing_whitespace());
    w.write("\u{00A0}".as_bytes());
    assert!(
        w.has_trailing_whitespace(),
        "no-break space is whitespace-like"
    );
    w.write(&[0xE2, 0x80]);
    assert!(
        !w.has_trailing_whitespace(),
        "a truncated sequence decodes as RuneError"
    );
    w.increase_indent();
    w.clear();
    assert_eq!((w.text(), w.get_indent(), w.get_line()), (&b""[..], 0, 0));
    assert!(w.is_at_start_of_line());
    w.write(b"a");
    assert_eq!(
        w.text(),
        b"a",
        "Clear resets the indent but keeps the newline and indent size"
    );
    w.write_line();
    assert_eq!(w.text(), b"a\n");
}

#[test]
#[should_panic(expected = "nonnegative indent")]
fn text_writer_keeps_upstream_panic_on_negative_indent() {
    let mut w = TextWriter::new(b"\n", 4);
    w.decrease_indent();
    w.write(b"a");
}

#[test]
fn single_line_writer_flattens_lines_and_reports_no_position() {
    let mut w = SingleLineStringWriter::new();
    w.write_keyword(b"type");
    w.write_line();
    w.increase_indent();
    w.write_punctuation(b"{");
    w.write_line_force(false);
    assert_eq!(w.text(), b"type { ");
    assert_eq!((w.get_line(), w.get_column(), w.get_indent()), (0, 0, 0));
    assert!(!w.is_at_start_of_line());
    assert!(!w.has_trailing_comment());
    assert!(w.has_trailing_whitespace());
    w.write(b"x");
    assert!(!w.has_trailing_whitespace());
    assert_eq!(w.get_text_pos(), 8);
    w.clear();
    assert_eq!(w.text(), b"");
    assert!(!w.has_trailing_whitespace());
}

#[test]
fn last_rune_follows_go_standard_decoding() {
    assert_eq!(decode_last_rune(b""), None);
    assert_eq!(decode_last_rune(b"ab"), Some('b'));
    assert_eq!(decode_last_rune("a\u{00A0}".as_bytes()), Some('\u{00A0}'));
    assert_eq!(decode_last_rune("\u{1F600}".as_bytes()), Some('\u{1F600}'));
    assert_eq!(decode_last_rune(&[0xE2, 0x80]), None, "truncated sequence");
    assert_eq!(decode_last_rune(&[0x80]), None, "lone continuation byte");
    assert_eq!(
        decode_last_rune(&[0xED, 0xA0, 0x80]),
        None,
        "surrogate encoding is invalid UTF-8"
    );
    assert_eq!(
        decode_last_rune("\u{FFFD}".as_bytes()),
        None,
        "matches the RuneError comparison"
    );
    assert_eq!(decode_last_rune(&[0x80, 0x80, 0x80, 0x80, 0x80]), None);
}
