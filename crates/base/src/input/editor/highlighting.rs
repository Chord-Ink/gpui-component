use std::{ops::Range, rc::Rc, sync::Arc};

use gpui::{AnyElement, Context, HighlightStyle, Hsla, SharedString, Window};
use ropey::Rope;

use super::{EditorState, FoldRange, InputEdit};
use crate::{CaretStyle, SemanticThemeTokens};

/// Resolves semantic highlight names into renderable GPUI styles.
///
/// Base deliberately knows nothing about a concrete syntax theme. UI crates and
/// applications can provide any resolver, independently of their parser.
pub trait HighlightStyleResolver: Send + Sync {
    fn style(&self, name: &str) -> Option<HighlightStyle>;
}

#[derive(Default)]
struct NoHighlightStyles;

impl HighlightStyleResolver for NoHighlightStyles {
    fn style(&self, _: &str) -> Option<HighlightStyle> {
        None
    }
}

/// Parser-independent syntax highlighting seam consumed by the Base editor.
///
/// Implementations own parsing, incremental state, and language-specific
/// behavior. Base only asks for styled ranges and fold candidates.
pub trait InputHighlighter {
    fn language(&self) -> SharedString;

    fn update(
        &mut self,
        edit: Option<InputEdit>,
        text: &Rope,
        folding: bool,
        window: &mut Window,
        cx: &mut Context<EditorState>,
    );

    /// Return ordered, non-overlapping style runs that fully cover `range`.
    /// Use [`HighlightStyle::default`] for text without a semantic style.
    fn styles(
        &self,
        range: &Range<usize>,
        resolver: &dyn HighlightStyleResolver,
    ) -> Vec<(Range<usize>, HighlightStyle)>;

    fn fold_ranges(&self, text: &Rope) -> Vec<FoldRange>;

    fn fold_ranges_for_edit(&self, range: Range<usize>, text: &Rope) -> Vec<FoldRange> {
        let _ = range;
        self.fold_ranges(text)
    }
}

pub type InputHighlighterFactory = Rc<dyn Fn(&str) -> Option<Box<dyn InputHighlighter>>>;
pub type SharedHighlightStyleResolver = Arc<dyn HighlightStyleResolver>;
pub type FoldIconRenderer = Rc<dyn Fn(usize, bool) -> AnyElement>;

/// The colors diagnostic underlines and popovers are drawn in, one per
/// severity.
#[derive(Clone, Copy, Default)]
pub struct DiagnosticColors {
    error: Hsla,
    warning: Hsla,
    info: Hsla,
    hint: Hsla,
}

impl DiagnosticColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_error(mut self, error: Hsla) -> Self {
        self.error = error;
        self
    }

    pub fn with_warning(mut self, warning: Hsla) -> Self {
        self.warning = warning;
        self
    }

    pub fn with_info(mut self, info: Hsla) -> Self {
        self.info = info;
        self
    }

    pub fn with_hint(mut self, hint: Hsla) -> Self {
        self.hint = hint;
        self
    }

    pub fn error(&self) -> Hsla {
        self.error
    }

    pub fn warning(&self) -> Hsla {
        self.warning
    }

    pub fn info(&self) -> Hsla {
        self.info
    }

    pub fn hint(&self) -> Hsla {
        self.hint
    }
}

/// Application-owned colors and highlight resolver consumed by editor painting.
#[derive(Clone)]
pub struct InputEditorStyle {
    foreground: Hsla,
    muted_foreground: Hsla,
    background: Hsla,
    border: Hsla,
    selection: Hsla,
    selection_foreground: Option<Hsla>,
    inactive_selection_opacity: f32,
    caret: CaretStyle,
    diagnostics: DiagnosticColors,
    highlight_styles: SharedHighlightStyleResolver,
    editor_invisible: Option<Hsla>,
    editor_active_line: Option<Hsla>,
    editor_gutter_background: Option<Hsla>,
    fold_icon_renderer: Option<FoldIconRenderer>,
}

impl InputEditorStyle {
    /// Fills in every colour that was left unset, from the active palette.
    ///
    /// `Hsla::default()` is fully transparent, and every colour on `Default` is
    /// that — so an input nothing projected onto painted its glyphs, its caret
    /// and its selection in nothing at all. Transparent is not a colour anyone
    /// means for ink, which is what makes it usable as "unset" here.
    ///
    /// This is resolution, not assignment: whatever a consumer did project is
    /// kept exactly. `crates/component` projects the whole style on every render and
    /// never reaches this; a consumer that projects once at construction gets
    /// the palette that is current now rather than the one that happened to be
    /// installed when the state was built.
    pub fn resolved(&self, tokens: &SemanticThemeTokens) -> Self {
        let colors = &tokens.colors;
        let unset = |value: Hsla| value.a == 0.;
        let or = |value: Hsla, fallback: Hsla| if unset(value) { fallback } else { value };

        let foreground = or(self.foreground, colors.foreground);
        let mut selection = self.selection;
        if unset(selection) {
            selection = colors.accent;
            // A selection must not hide the glyphs it selects.
            selection.a = 0.4;
        }

        Self {
            foreground,
            muted_foreground: or(self.muted_foreground, colors.muted_foreground),
            background: or(self.background, colors.surface),
            border: or(self.border, colors.border),
            selection,
            caret: self.caret.with_color(or(self.caret.color(), foreground)),
            ..self.clone()
        }
    }
}

impl Default for InputEditorStyle {
    fn default() -> Self {
        Self {
            foreground: Hsla::default(),
            muted_foreground: Hsla::default(),
            background: Hsla::default(),
            border: Hsla::default(),
            selection: Hsla::default(),
            selection_foreground: None,
            inactive_selection_opacity: 1.,
            caret: CaretStyle::default(),
            diagnostics: DiagnosticColors::default(),
            highlight_styles: Arc::new(NoHighlightStyles),
            editor_invisible: None,
            editor_active_line: None,
            editor_gutter_background: None,
            fold_icon_renderer: None,
        }
    }
}

impl InputEditorStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_foreground(mut self, foreground: Hsla) -> Self {
        self.foreground = foreground;
        self
    }

    pub fn with_muted_foreground(mut self, muted_foreground: Hsla) -> Self {
        self.muted_foreground = muted_foreground;
        self
    }

    pub fn with_background(mut self, background: Hsla) -> Self {
        self.background = background;
        self
    }

    pub fn with_border(mut self, border: Hsla) -> Self {
        self.border = border;
        self
    }

    pub fn with_selection(mut self, selection: Hsla) -> Self {
        self.selection = selection;
        self
    }

    /// The color selected text is recolored to, or `None` to leave every glyph
    /// its own color.
    ///
    /// Windows, GTK3 and Qt all flip selected text to white, but each pairs
    /// that with an opaque fill, where the flip is what keeps the text
    /// readable. Under a translucent selection it flattens whatever coloring
    /// the text already carried — syntax, diagnostics, a diff — so `None` is
    /// the default and the styled layer opts in.
    pub fn with_selection_foreground(mut self, selection_foreground: Option<Hsla>) -> Self {
        self.selection_foreground = selection_foreground;
        self
    }

    /// How far the selection dims while the window is not the active one.
    ///
    /// A selection that vanished on deactivation would leave the reader unable
    /// to see what they had selected when they came back, so every desktop
    /// dims or greys it instead; `1.0` leaves it at full strength.
    pub fn with_inactive_selection_opacity(mut self, opacity: f32) -> Self {
        self.inactive_selection_opacity = opacity;
        self
    }

    /// How the insertion caret is drawn — its width, cap radius and color.
    pub fn with_caret(mut self, caret: CaretStyle) -> Self {
        self.caret = caret;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: DiagnosticColors) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_highlight_styles(mut self, highlight_styles: SharedHighlightStyleResolver) -> Self {
        self.highlight_styles = highlight_styles;
        self
    }

    /// The color whitespace indicators are drawn in. `None` leaves them unpainted.
    pub fn with_editor_invisible(mut self, editor_invisible: Option<Hsla>) -> Self {
        self.editor_invisible = editor_invisible;
        self
    }

    /// The background behind the line the caret sits on. `None` leaves it unpainted.
    pub fn with_editor_active_line(mut self, editor_active_line: Option<Hsla>) -> Self {
        self.editor_active_line = editor_active_line;
        self
    }

    /// The background behind the line-number gutter. `None` falls back to the
    /// editor background.
    pub fn with_editor_gutter_background(mut self, editor_gutter_background: Option<Hsla>) -> Self {
        self.editor_gutter_background = editor_gutter_background;
        self
    }

    /// Draws the fold arrow beside a foldable line. `None` leaves folds
    /// keyboard-only.
    pub fn with_fold_icon_renderer(mut self, fold_icon_renderer: Option<FoldIconRenderer>) -> Self {
        self.fold_icon_renderer = fold_icon_renderer;
        self
    }

    pub fn foreground(&self) -> Hsla {
        self.foreground
    }

    pub fn muted_foreground(&self) -> Hsla {
        self.muted_foreground
    }

    pub fn background(&self) -> Hsla {
        self.background
    }

    pub fn border(&self) -> Hsla {
        self.border
    }

    pub fn selection(&self) -> Hsla {
        self.selection
    }

    pub fn selection_foreground(&self) -> Option<Hsla> {
        self.selection_foreground
    }

    pub fn inactive_selection_opacity(&self) -> f32 {
        self.inactive_selection_opacity
    }

    pub fn caret(&self) -> CaretStyle {
        self.caret
    }

    pub fn diagnostics(&self) -> DiagnosticColors {
        self.diagnostics
    }

    pub fn highlight_styles(&self) -> &SharedHighlightStyleResolver {
        &self.highlight_styles
    }

    pub fn editor_invisible(&self) -> Option<Hsla> {
        self.editor_invisible
    }

    pub fn editor_active_line(&self) -> Option<Hsla> {
        self.editor_active_line
    }

    pub fn editor_gutter_background(&self) -> Option<Hsla> {
        self.editor_gutter_background
    }

    pub fn fold_icon_renderer(&self) -> Option<&FoldIconRenderer> {
        self.fold_icon_renderer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::InputEditorStyle;
    use crate::{CaretStyle, SemanticThemeTokens};

    fn dark() -> SemanticThemeTokens {
        let mut tokens = SemanticThemeTokens::default();
        tokens.colors.foreground = hsla(0., 0., 0.98, 1.0);
        tokens.colors.muted_foreground = hsla(0., 0., 0.64, 1.0);
        tokens.colors.surface = hsla(0., 0., 0.04, 1.0);
        tokens.colors.border = hsla(0., 0., 0.15, 1.0);
        tokens.colors.accent = hsla(0.6, 0.5, 0.5, 1.0);
        tokens
    }

    #[test]
    fn an_unprojected_style_takes_its_ink_from_the_palette() {
        let tokens = dark();
        let resolved = InputEditorStyle::new().resolved(&tokens);

        assert_eq!(resolved.foreground(), tokens.colors.foreground);
        assert_eq!(resolved.caret().color(), tokens.colors.foreground);
        assert_eq!(resolved.muted_foreground(), tokens.colors.muted_foreground);
        assert_eq!(resolved.background(), tokens.colors.surface);
        assert_eq!(resolved.border(), tokens.colors.border);
        // The point of the change: every one of these was transparent, so an
        // input nothing projected onto painted its text in nothing at all.
        for colour in [
            resolved.foreground(),
            resolved.caret().color(),
            resolved.muted_foreground(),
            resolved.selection(),
        ] {
            assert!(colour.a > 0., "{colour:?} is still invisible");
        }
    }

    #[test]
    fn a_selection_stays_translucent_enough_to_read_through() {
        let resolved = InputEditorStyle::new().resolved(&dark());
        assert_eq!(resolved.selection().a, 0.4);
    }

    #[test]
    fn projected_colours_are_kept_verbatim() {
        let chosen = hsla(0.3, 0.4, 0.5, 1.0);
        let resolved = InputEditorStyle::new()
            .with_foreground(chosen)
            .with_caret(CaretStyle::new().with_color(chosen))
            .resolved(&dark());

        assert_eq!(resolved.foreground(), chosen);
        assert_eq!(resolved.caret().color(), chosen);
        // And what was not projected still comes from the palette.
        assert_eq!(resolved.border(), dark().colors.border);
    }

    #[test]
    fn resolution_never_consumes_its_own_output() {
        // The projected style is kept verbatim precisely so that this holds:
        // resolving against a second palette must follow it, not stay on the
        // first. Resolving in place would have frozen after one pass.
        let projected = InputEditorStyle::new();
        let first = projected.resolved(&dark());

        let mut light = SemanticThemeTokens::default();
        light.colors.foreground = hsla(0., 0., 0.04, 1.0);
        let second = projected.resolved(&light);

        assert_ne!(first.foreground(), second.foreground());
        assert_eq!(second.foreground(), light.colors.foreground);
    }
}
