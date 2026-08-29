use gpui::{App, Global};

use crate::{CaretMotion, ScrollbarMode, ScrollbarMotion, ScrollbarStyles, SemanticThemeTokens};

/// Application-wide defaults for Base behavior modules.
///
/// The fields are private and reached through the methods below, so the styled
/// layer can hand Base another kind of default — the caret's blink timing was
/// the last one — without the addition breaking every application that builds
/// a theme.
#[derive(Clone, Default)]
pub struct Theme {
    tokens: SemanticThemeTokens,
    scrollbar: ScrollbarTheme,
    resizable: ResizableTheme,
    caret_motion: CaretMotion,
}

impl Global for Theme {}

impl Theme {
    pub fn new() -> Self {
        Self::default()
    }

    /// The semantic colors, spacing, radii and type Base reads when it paints.
    pub fn with_tokens(mut self, tokens: SemanticThemeTokens) -> Self {
        self.tokens = tokens;
        self
    }

    pub fn with_scrollbar(mut self, scrollbar: ScrollbarTheme) -> Self {
        self.scrollbar = scrollbar;
        self
    }

    pub fn with_resizable(mut self, resizable: ResizableTheme) -> Self {
        self.resizable = resizable;
        self
    }

    /// Timing for the caret every input and OTP field blinks. The caret's own
    /// width, radius and color travel with the field that draws it; only the
    /// blink is application-wide, because one timer serves every field.
    pub fn with_caret_motion(mut self, caret_motion: CaretMotion) -> Self {
        self.caret_motion = caret_motion;
        self
    }

    pub fn tokens(&self) -> &SemanticThemeTokens {
        &self.tokens
    }

    pub fn scrollbar(&self) -> &ScrollbarTheme {
        &self.scrollbar
    }

    pub fn resizable(&self) -> ResizableTheme {
        self.resizable
    }

    pub fn caret_motion(&self) -> CaretMotion {
        self.caret_motion
    }

    pub fn global(cx: &App) -> Self {
        cx.try_global::<Self>().cloned().unwrap_or_default()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        if !cx.has_global::<Self>() {
            cx.set_global(Self::default());
        }
        cx.global_mut::<Self>()
    }

    /// Rebuilds the global theme through the builder.
    ///
    /// The fields are private, so a caller changes one part by handing the
    /// theme back with that part replaced rather than assigning into it:
    ///
    /// ```
    /// use gpui_base::{CaretMotion, Theme};
    /// # fn example(cx: &mut gpui::App) {
    /// Theme::update(cx, |theme| {
    ///     theme.with_caret_motion(CaretMotion::new().with_blinking(false))
    /// });
    /// # }
    /// ```
    ///
    /// A part that is itself a builder is read, rebuilt, and put back:
    ///
    /// ```
    /// use gpui_base::{ScrollbarMode, Theme};
    /// # fn example(cx: &mut gpui::App) {
    /// Theme::update(cx, |theme| {
    ///     let scrollbar = theme.scrollbar().clone().with_mode(ScrollbarMode::Hover);
    ///     theme.with_scrollbar(scrollbar)
    /// });
    /// # }
    /// ```
    pub fn update(cx: &mut App, build: impl FnOnce(Self) -> Self) {
        let theme = build(Self::global(cx));
        *Self::global_mut(cx) = theme;
    }
}

/// Access to the active base theme through an application context.
pub(crate) trait ActiveTheme {
    fn theme(&self) -> Theme;
}

impl ActiveTheme for App {
    #[inline(always)]
    fn theme(&self) -> Theme {
        Theme::global(self)
    }
}

/// Global defaults used by [`crate::Scrollbar`].
///
/// `motion` defaults to motionless. Styled layers project their own timing;
/// Base never installs a fade or slide of its own.
#[derive(Clone, Default)]
pub struct ScrollbarTheme {
    mode: ScrollbarMode,
    motion: ScrollbarMotion,
    styles: ScrollbarStyles,
}

impl ScrollbarTheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mode(mut self, mode: ScrollbarMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_motion(mut self, motion: ScrollbarMotion) -> Self {
        self.motion = motion;
        self
    }

    pub fn with_styles(mut self, styles: ScrollbarStyles) -> Self {
        self.styles = styles;
        self
    }

    pub fn mode(&self) -> ScrollbarMode {
        self.mode
    }

    pub fn motion(&self) -> ScrollbarMotion {
        self.motion
    }

    pub fn styles(&self) -> &ScrollbarStyles {
        &self.styles
    }
}

/// Global visual defaults used by resizable panel handles.
///
/// The Base default is transparent. Applications and styled façades may
/// project their own colors without coupling resize behavior to a theme crate.
#[derive(Clone, Copy, Default)]
pub struct ResizableTheme {
    handle: gpui::Hsla,
    active_handle: gpui::Hsla,
}

impl ResizableTheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_handle(mut self, handle: gpui::Hsla) -> Self {
        self.handle = handle;
        self
    }

    /// The handle under the pointer that is dragging it.
    pub fn with_active_handle(mut self, active_handle: gpui::Hsla) -> Self {
        self.active_handle = active_handle;
        self
    }

    pub fn handle(&self) -> gpui::Hsla {
        self.handle
    }

    pub fn active_handle(&self) -> gpui::Hsla {
        self.active_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_supports_builder_construction() {
        let motion = CaretMotion::new().with_blinking(false);
        let resizable = ResizableTheme::new().with_handle(gpui::red());
        let theme = Theme::new()
            .with_scrollbar(ScrollbarTheme::new().with_mode(ScrollbarMode::Hover))
            .with_resizable(resizable)
            .with_caret_motion(motion);

        assert_eq!(theme.scrollbar().mode(), ScrollbarMode::Hover);
        assert_eq!(theme.resizable().handle(), gpui::red());
        assert_eq!(theme.caret_motion(), motion);
        let _ = theme.tokens();
    }

    #[test]
    fn resizable_theme_supports_builder_construction() {
        let theme = ResizableTheme::new()
            .with_handle(gpui::red())
            .with_active_handle(gpui::blue());

        assert_eq!(theme.handle(), gpui::red());
        assert_eq!(theme.active_handle(), gpui::blue());
    }

    #[test]
    fn scrollbar_theme_supports_builder_construction() {
        let mode = ScrollbarMode::Hover;
        let motion = ScrollbarMotion::default().with_enter(std::time::Duration::from_millis(120));
        let styles = ScrollbarStyles::default();
        let theme = ScrollbarTheme::new()
            .with_mode(mode)
            .with_motion(motion)
            .with_styles(styles);

        assert_eq!(theme.mode(), mode);
        assert_eq!(theme.motion(), motion);
        let _ = theme.styles();
    }
}
