use gpui::{App, Global};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CaretMotion, ScrollbarMode, ScrollbarMotion, ScrollbarStyles, SemanticThemeTokens};

/// Application-wide defaults for Base behavior modules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemeAppearance {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Default)]
pub struct Theme {
    appearance: ThemeAppearance,
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

    pub fn with_appearance(mut self, appearance: ThemeAppearance) -> Self {
        self.appearance = appearance;
        self
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

    /// Timing for the caret every input and OTP field blinks. One timer serves
    /// every field, so only the blink is application-wide.
    pub fn with_caret_motion(mut self, caret_motion: CaretMotion) -> Self {
        self.caret_motion = caret_motion;
        self
    }

    pub fn appearance(&self) -> ThemeAppearance {
        self.appearance
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
/// `motion` defaults to motionless; styled layers project their own timing.
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
/// `None` means *unset*, not invisible: a handle with nothing projected onto it
/// resolves from the active [`SemanticThemeTokens`] -- `border` at rest, `ring`
/// while dragging -- which are the tokens those two states already mean
/// everywhere else.
///
/// These were plain colors, so the Base default was `Hsla::default()`: fully
/// transparent. That reads as a deliberate choice next to a styled façade,
/// which projects its own values and never sees it, and as a missing divider
/// to anything that does not -- and a consumer with no façade has no way to
/// project anything. Making them optional keeps the projection exactly as it
/// was while giving the unprojected case an answer.
#[derive(Clone, Copy, Default)]
pub struct ResizableTheme {
    handle: Option<gpui::Hsla>,
    active_handle: Option<gpui::Hsla>,
}

impl ResizableTheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_handle(mut self, handle: gpui::Hsla) -> Self {
        self.handle = Some(handle);
        self
    }

    /// The handle under the pointer that is dragging it.
    pub fn with_active_handle(mut self, active_handle: gpui::Hsla) -> Self {
        self.active_handle = Some(active_handle);
        self
    }

    pub fn handle(&self) -> Option<gpui::Hsla> {
        self.handle
    }

    pub fn active_handle(&self) -> Option<gpui::Hsla> {
        self.active_handle
    }
}
