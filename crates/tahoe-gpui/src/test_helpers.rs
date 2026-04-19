//! Shared test utilities for tahoe-gpui components.
//!
//! Provides common assertions and fixtures to reduce test boilerplate.

#[cfg(test)]
pub(crate) mod helpers {
    use crate::components::selection_and_input::text_field::TEXT_FIELD_CONTEXT;
    use crate::foundations::color::contrast_ratio;
    use crate::foundations::theme::TahoeTheme;
    use crate::text_actions::{
        Backspace, Copy, Cut, Delete, End, Home, Left, Paste, Right, SelectAll, SelectLeft,
        SelectRight, SelectWordLeft, SelectWordRight, WordLeft, WordRight,
    };
    use gpui::{
        Bounds, Entity, Hsla, KeyBinding, Modifiers, MouseButton, Pixels, Point, Render,
        TestAppContext, VisualTestContext, Window, point,
    };

    /// Create a light theme for testing.
    pub fn theme_light() -> TahoeTheme {
        TahoeTheme::light()
    }

    /// Create a dark theme for testing.
    pub fn theme_dark() -> TahoeTheme {
        TahoeTheme::dark()
    }

    /// Assert that foreground/background meet WCAG AA contrast (4.5:1).
    pub fn assert_contrast_aa(fg: Hsla, bg: Hsla, context: &str) {
        let ratio = contrast_ratio(fg, bg);
        assert!(
            ratio >= 4.5,
            "{context}: contrast {ratio:.2}:1 fails WCAG AA (need 4.5:1)"
        );
    }

    /// Assert that foreground/background meet WCAG AAA contrast (7.0:1).
    pub fn assert_contrast_aaa(fg: Hsla, bg: Hsla, context: &str) {
        let ratio = contrast_ratio(fg, bg);
        assert!(
            ratio >= 7.0,
            "{context}: contrast {ratio:.2}:1 fails WCAG AAA (need 7.0:1)"
        );
    }

    /// Assert all variants of an enum are distinct from each other.
    pub fn assert_all_distinct<T: PartialEq + std::fmt::Debug>(variants: &[T]) {
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "variants at index {i} and {j} should be distinct");
                }
            }
        }
    }

    /// Keybindings used by `TextField`'s [`TEXT_FIELD_CONTEXT`] key context.
    ///
    /// The scope string is always sourced from [`TEXT_FIELD_CONTEXT`] so the
    /// test bindings stay in lock-step with the production context string —
    /// renaming the constant propagates here automatically.
    pub fn text_field_keybindings() -> Vec<KeyBinding> {
        let ctx = Some(TEXT_FIELD_CONTEXT);
        vec![
            KeyBinding::new("backspace", Backspace, ctx),
            KeyBinding::new("delete", Delete, ctx),
            KeyBinding::new("left", Left, ctx),
            KeyBinding::new("right", Right, ctx),
            KeyBinding::new("shift-left", SelectLeft, ctx),
            KeyBinding::new("shift-right", SelectRight, ctx),
            KeyBinding::new("cmd-a", SelectAll, ctx),
            KeyBinding::new("home", Home, ctx),
            KeyBinding::new("cmd-left", Home, ctx),
            KeyBinding::new("end", End, ctx),
            KeyBinding::new("cmd-right", End, ctx),
            KeyBinding::new("alt-left", WordLeft, ctx),
            KeyBinding::new("alt-right", WordRight, ctx),
            KeyBinding::new("alt-shift-left", SelectWordLeft, ctx),
            KeyBinding::new("alt-shift-right", SelectWordRight, ctx),
            KeyBinding::new("cmd-c", Copy, ctx),
            KeyBinding::new("cmd-x", Cut, ctx),
            KeyBinding::new("cmd-v", Paste, ctx),
        ]
    }

    /// Register text field keybindings for tests that simulate keyboard input.
    pub fn register_test_keybindings(cx: &mut TestAppContext) {
        cx.update(|cx| {
            cx.bind_keys(text_field_keybindings());
        });
    }

    /// Create a test window with dark theme and standard interaction keybindings.
    pub fn setup_test_window<V: Render + 'static>(
        cx: &mut TestAppContext,
        build: impl FnOnce(&mut Window, &mut gpui::Context<V>) -> V,
    ) -> (Entity<V>, &mut VisualTestContext) {
        register_test_keybindings(cx);
        cx.add_window_view(|window, cx| {
            cx.set_global(TahoeTheme::dark());
            build(window, cx)
        })
    }

    /// Create a test window with light theme and standard interaction keybindings.
    #[allow(dead_code)] // Parity helper for future light-mode tests.
    pub fn setup_test_window_light<V: Render + 'static>(
        cx: &mut TestAppContext,
        build: impl FnOnce(&mut Window, &mut gpui::Context<V>) -> V,
    ) -> (Entity<V>, &mut VisualTestContext) {
        register_test_keybindings(cx);
        cx.add_window_view(|window, cx| {
            cx.set_global(TahoeTheme::light());
            build(window, cx)
        })
    }

    /// A rendered element found via `debug_bounds`.
    #[derive(Debug, Clone, Copy)]
    pub struct LocatedElement {
        pub bounds: Bounds<Pixels>,
    }

    impl LocatedElement {
        /// The center point of the element.
        pub fn center(&self) -> Point<Pixels> {
            self.bounds.center()
        }

        /// A point at the given fractions within the element's bounds.
        pub fn point_at(&self, x_fraction: f32, y_fraction: f32) -> Point<Pixels> {
            let x_fraction = x_fraction.clamp(0.0, 1.0);
            let y_fraction = y_fraction.clamp(0.0, 1.0);
            point(
                self.bounds.left() + self.bounds.size.width * x_fraction,
                self.bounds.top() + self.bounds.size.height * y_fraction,
            )
        }
    }

    /// Locator helpers backed by GPUI debug selectors.
    pub trait LocatorExt {
        fn find_element(&mut self, selector: &'static str) -> Option<LocatedElement>;
        fn get_element(&mut self, selector: &'static str) -> LocatedElement;
        fn has_element(&mut self, selector: &'static str) -> bool;
    }

    impl LocatorExt for VisualTestContext {
        fn find_element(&mut self, selector: &'static str) -> Option<LocatedElement> {
            self.debug_bounds(selector)
                .map(|bounds| LocatedElement { bounds })
        }

        fn get_element(&mut self, selector: &'static str) -> LocatedElement {
            self.find_element(selector).unwrap_or_else(|| {
                panic!(
                    "Element '{selector}' not found in rendered frame. Hint: add .debug_selector(|| \"{selector}\".into())"
                )
            })
        }

        fn has_element(&mut self, selector: &'static str) -> bool {
            self.find_element(selector).is_some()
        }
    }

    /// Small interaction helpers used by component tests.
    pub trait InteractionExt: LocatorExt {
        fn click_on(&mut self, selector: &'static str);
        fn click_at(&mut self, position: Point<Pixels>);
        fn click_within(&mut self, selector: &'static str, x_fraction: f32, y_fraction: f32);
        fn drag_between_points(&mut self, from: Point<Pixels>, to: Point<Pixels>);
        fn drag_within_x(&mut self, selector: &'static str, from_fraction: f32, to_fraction: f32);
        fn type_text(&mut self, text: &str);
        fn press(&mut self, keystroke: &str);
    }

    impl InteractionExt for VisualTestContext {
        fn click_on(&mut self, selector: &'static str) {
            let element = self.get_element(selector);
            self.simulate_click(element.center(), Modifiers::default());
        }

        fn click_at(&mut self, position: Point<Pixels>) {
            self.simulate_click(position, Modifiers::default());
        }

        fn click_within(&mut self, selector: &'static str, x_fraction: f32, y_fraction: f32) {
            let element = self.get_element(selector);
            self.click_at(element.point_at(x_fraction, y_fraction));
        }

        fn drag_between_points(&mut self, from: Point<Pixels>, to: Point<Pixels>) {
            self.simulate_mouse_down(from, MouseButton::Left, Modifiers::default());
            self.simulate_mouse_move(to, MouseButton::Left, Modifiers::default());
            self.simulate_mouse_up(to, MouseButton::Left, Modifiers::default());
        }

        fn drag_within_x(&mut self, selector: &'static str, from_fraction: f32, to_fraction: f32) {
            let element = self.get_element(selector);
            let start = element.point_at(from_fraction, 0.5);
            let end = element.point_at(to_fraction, 0.5);
            self.drag_between_points(start, end);
        }

        fn type_text(&mut self, text: &str) {
            self.simulate_input(text);
        }

        fn press(&mut self, keystroke: &str) {
            self.simulate_keystrokes(keystroke);
        }
    }

    pub fn assert_element_exists(cx: &mut VisualTestContext, selector: &'static str) {
        assert!(
            cx.has_element(selector),
            "Expected element '{selector}' to exist in the rendered frame"
        );
    }

    pub fn assert_element_absent(cx: &mut VisualTestContext, selector: &'static str) {
        assert!(
            !cx.has_element(selector),
            "Expected element '{selector}' to be absent from the rendered frame"
        );
    }

    #[cfg(test)]
    mod self_tests {
        use super::{
            assert_all_distinct, assert_contrast_aa, assert_contrast_aaa, theme_dark, theme_light,
        };
        use core::prelude::v1::test;

        // ── Theme helpers produce distinct values ────────────────────

        #[test]
        fn theme_light_differs_from_theme_dark() {
            let light = theme_light();
            let dark = theme_dark();
            assert_ne!(light.text, dark.text);
            assert_ne!(light.background, dark.background);
        }

        // ── Contrast helpers: primary text on primary background ─────

        #[test]
        fn light_theme_primary_text_meets_wcag_aa() {
            let theme = theme_light();
            assert_contrast_aa(theme.text, theme.background, "light theme text on background");
        }

        #[test]
        fn dark_theme_primary_text_meets_wcag_aa() {
            let theme = theme_dark();
            assert_contrast_aa(theme.text, theme.background, "dark theme text on background");
        }

        #[test]
        fn light_theme_primary_text_meets_wcag_aaa() {
            // HIG accessibility recommends AAA (7:1) for primary body
            // text. Guards against future theme edits that would lower
            // the contrast of the primary reading surface.
            let theme = theme_light();
            assert_contrast_aaa(theme.text, theme.background, "light theme text on background");
        }

        #[test]
        fn dark_theme_primary_text_meets_wcag_aaa() {
            let theme = theme_dark();
            assert_contrast_aaa(theme.text, theme.background, "dark theme text on background");
        }

        // ── assert_all_distinct ──────────────────────────────────────

        #[test]
        fn assert_all_distinct_passes_for_distinct_values() {
            assert_all_distinct(&[1, 2, 3, 4]);
        }

        #[test]
        #[should_panic(expected = "should be distinct")]
        fn assert_all_distinct_panics_on_duplicates() {
            assert_all_distinct(&[1, 2, 2, 4]);
        }
    }
}
