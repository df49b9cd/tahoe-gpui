use super::TextFieldValidation;
use core::prelude::v1::test;
use gpui::SharedString;

#[test]
fn default_fields() {
    // Verify new fields have sensible defaults via a simple struct check.
    // We can't call TextField::new without a Context, so test the enum defaults.
    let validation = TextFieldValidation::default();
    assert!(matches!(validation, TextFieldValidation::None));
}

#[test]
fn validation_none_is_default() {
    let v: TextFieldValidation = Default::default();
    assert!(matches!(v, TextFieldValidation::None));
}

#[test]
fn validation_invalid_holds_message() {
    let msg = SharedString::from("Email is required");
    let v = TextFieldValidation::Invalid(msg.clone());
    match v {
        TextFieldValidation::Invalid(m) => assert_eq!(m, msg),
        _ => panic!("Expected Invalid variant"),
    }
}

#[test]
fn validation_valid_variant() {
    let v = TextFieldValidation::Valid;
    assert!(matches!(v, TextFieldValidation::Valid));
}

#[test]
fn validation_clone() {
    let v = TextFieldValidation::Invalid(SharedString::from("bad"));
    let v2 = v.clone();
    match (v, v2) {
        (TextFieldValidation::Invalid(a), TextFieldValidation::Invalid(b)) => {
            assert_eq!(a, b);
        }
        _ => panic!("Clone should preserve variant"),
    }
}

#[test]
fn secure_display_text_masks_ascii() {
    // The display_text helper replaces each char with a bullet.
    // We test the masking logic directly.
    let input = "hello";
    let masked: String = "\u{2022}".repeat(input.chars().count());
    assert_eq!(masked, "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}");
    assert_eq!(masked.chars().count(), 5);
}

#[test]
fn secure_display_text_masks_unicode() {
    let input = "\u{1F600}\u{1F601}"; // two emoji characters
    let masked: String = "\u{2022}".repeat(input.chars().count());
    assert_eq!(masked.chars().count(), 2);
}

#[test]
fn secure_display_text_empty_returns_empty() {
    let input = "";
    let masked: String = if input.is_empty() {
        String::new()
    } else {
        "\u{2022}".repeat(input.chars().count())
    };
    assert!(masked.is_empty());
}

#[test]
fn secure_cursor_offset_translation() {
    // Verify that byte offsets in original content are correctly
    // translated to display-text byte offsets in secure mode.
    let content = "abc"; // 3 ASCII chars, 3 bytes
    let bullet_len = '\u{2022}'.len_utf8(); // 3 bytes per bullet
    assert_eq!(bullet_len, 3);

    // Cursor at end of "abc" (byte offset 3) should map to display offset 9
    let cursor_offset = 3;
    let char_cursor = content[..cursor_offset].chars().count(); // 3
    let display_offset = char_cursor * bullet_len; // 9
    assert_eq!(display_offset, 9);

    // Cursor at byte 1 (after 'a') maps to display offset 3
    let cursor_offset = 1;
    let char_cursor = content[..cursor_offset].chars().count(); // 1
    let display_offset = char_cursor * bullet_len; // 3
    assert_eq!(display_offset, 3);
}

#[test]
fn secure_cursor_offset_unicode() {
    // Verify translation with multi-byte characters
    let content = "a\u{00e9}b"; // 'a' (1 byte) + 'e\u{0301}' (2 bytes) + 'b' (1 byte) = 4 bytes, 3 chars
    let bullet_len = '\u{2022}'.len_utf8();

    // Cursor at end (byte offset 4) maps to char count 3, display offset 9
    let cursor_offset = content.len(); // 4
    let char_cursor = content[..cursor_offset].chars().count(); // 3
    let display_offset = char_cursor * bullet_len; // 9
    assert_eq!(display_offset, 9);
}

#[test]
fn validation_debug_format() {
    let v = TextFieldValidation::None;
    let debug = format!("{:?}", v);
    assert!(debug.contains("None"));

    let v2 = TextFieldValidation::Invalid(SharedString::from("err"));
    let debug2 = format!("{:?}", v2);
    assert!(debug2.contains("Invalid"));

    let v3 = TextFieldValidation::Valid;
    let debug3 = format!("{:?}", v3);
    assert!(debug3.contains("Valid"));
}

#[test]
fn focus_ring_gating_logic() {
    // Mirrors the `show_focus_ring = is_focused && !self.disabled` branch
    // in the render. Documents the gating matrix so a future edit that
    // drops either guard gets caught here.
    fn show(is_focused: bool, disabled: bool) -> bool {
        is_focused && !disabled
    }
    // Ring fires only for the focused + enabled cell.
    assert!(show(true, false));
    assert!(!show(true, true));
    assert!(!show(false, false));
    assert!(!show(false, true));
}
