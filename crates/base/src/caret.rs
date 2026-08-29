use std::time::Duration;

use gpui::{Half as _, Hsla, Pixels, Size, Window, px};

/// How the text insertion caret is drawn.
///
/// Base paints the caret and measures the horizontal scroll against it, so the
/// width is geometry as much as appearance; it owns none of these values. The
/// defaults are the caret a browser draws — one pixel, square, and the text
/// color — and the styled layer supplies the platform's own.
///
/// ```
/// use gpui::px;
/// use gpui_base::CaretStyle;
///
/// let caret = CaretStyle::new().with_width(px(2.)).with_radius(px(1.));
///
/// assert_eq!(caret.width(), px(2.));
/// assert_eq!(caret.radius(), px(1.));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretStyle {
    width: Pixels,
    radius: Pixels,
    color: Hsla,
}

impl Default for CaretStyle {
    fn default() -> Self {
        Self {
            // One pixel: what Blink and Gecko draw, and the width Windows,
            // GTK and Qt each report as their own. macOS asks for two, and
            // says so from the styled layer.
            width: px(1.),
            // Square. A rounded cap is a platform's choice, not Base's.
            radius: px(0.),
            color: Hsla::default(),
        }
    }
}

impl CaretStyle {
    pub fn new() -> Self {
        Self::default()
    }

    /// How wide the caret is painted. Horizontal scrolling and the right-align
    /// clamp are measured against it, so this is geometry as well as color.
    pub fn with_width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// The radius of both caps, clamped at paint to half the caret's shorter
    /// side. Zero is a square caret. Derive it from the theme rather than
    /// writing a literal, so a theme that squares its corners squares this too.
    pub fn with_radius(mut self, radius: Pixels) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    pub fn width(&self) -> Pixels {
        self.width
    }

    pub fn radius(&self) -> Pixels {
        self.radius
    }

    pub fn color(&self) -> Hsla {
        self.color
    }

    /// How far left of the character boundary the caret is drawn.
    ///
    /// A caret wider than a pixel straddles the boundary instead of covering
    /// the glyph after it, which is what every browser engine does. A
    /// single-pixel caret sits on the boundary, where a half-pixel shift would
    /// only blur it.
    pub fn boundary_offset(&self) -> Pixels {
        if self.width > px(1.) {
            self.width.half()
        } else {
            px(0.)
        }
    }

    /// The radius the caret can actually carry inside `bounds`, since a quad
    /// painted with an oversized radius lets its corners overrun each other.
    pub fn clamped_radius(&self, size: Size<Pixels>) -> Pixels {
        self.radius.min(size.width.half()).min(size.height.half())
    }
}

/// Ascent plus descent of the window's current text face at `font_size` — the
/// height a caret takes from its font, before it is rounded to a whole pixel.
///
/// GPUI reports a font's `descent` the way OpenType stores it, negative below
/// the baseline, while a shaped line reports it already flipped positive. The
/// subtraction here is what makes a caret measured from the face and one
/// measured from shaped text come out the same height.
pub fn font_caret_height(font_size: Pixels, window: &Window) -> Pixels {
    let text_system = window.text_system();
    let font_id = text_system.resolve_font(&window.text_style().font());

    text_system.ascent(font_id, font_size) - text_system.descent(font_id, font_size)
}

/// The shortest blink a caret is allowed, one frame at 60Hz. Anything less
/// reads as a steady caret on screen while still costing a timer.
const MIN_INTERVAL: Duration = Duration::from_millis(16);

/// How the text insertion caret blinks.
///
/// Base owns the timer, because the blink is bound to focus, typing and window
/// activation, and owns none of its timing. Every desktop platform blinks its
/// caret unless the user has asked it not to, so the default does too.
///
/// ```
/// use std::time::Duration;
/// use gpui_base::CaretMotion;
///
/// let steady = CaretMotion::new().with_blinking(false);
///
/// assert!(!steady.is_blinking());
/// assert_eq!(steady.interval(), Duration::from_millis(500));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretMotion {
    blinking: bool,
    interval: Duration,
    pause: Duration,
}

impl Default for CaretMotion {
    fn default() -> Self {
        Self {
            blinking: true,
            // Half a second in each phase: Blink, Gecko, WinUI's own fallback
            // and every editor that had to pick a number land here.
            interval: Duration::from_millis(500),
            // One full phase, so a burst of keystrokes never starts a blink
            // mid-word. GTK holds for 600ms, Qt and JetBrains for one phase.
            pause: Duration::from_millis(500),
        }
    }
}

impl CaretMotion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the caret blinks at all. A steady caret still shows and hides
    /// with focus; it simply never turns itself off while the field is active.
    pub fn with_blinking(mut self, blinking: bool) -> Self {
        self.blinking = blinking;
        self
    }

    /// How long the caret rests in each of its two phases.
    ///
    /// Floored at a frame: a caret cannot blink faster than the screen can
    /// show it, and a zero interval would leave the blink timer with nothing
    /// to wait for. Hold the caret still with [`Self::with_blinking`] instead.
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval.max(MIN_INTERVAL);
        self
    }

    /// How long the caret is held visible after a keystroke before it resumes.
    pub fn with_pause(mut self, pause: Duration) -> Self {
        self.pause = pause;
        self
    }

    pub fn is_blinking(&self) -> bool {
        self.blinking
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn pause(&self) -> Duration {
        self.pause
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{IntoElement, size};

    #[test]
    fn caret_style_supports_builder_construction() {
        let style = CaretStyle::new()
            .with_width(px(2.))
            .with_radius(px(1.))
            .with_color(gpui::red());

        assert_eq!(style.width(), px(2.));
        assert_eq!(style.radius(), px(1.));
        assert_eq!(style.color(), gpui::red());
    }

    #[test]
    fn caret_motion_supports_builder_construction() {
        let motion = CaretMotion::new()
            .with_blinking(false)
            .with_interval(Duration::from_millis(120))
            .with_pause(Duration::from_millis(240));

        assert!(!motion.is_blinking());
        assert_eq!(motion.interval(), Duration::from_millis(120));
        assert_eq!(motion.pause(), Duration::from_millis(240));
    }

    #[test]
    fn only_a_caret_wider_than_a_pixel_straddles_the_boundary() {
        assert_eq!(CaretStyle::new().boundary_offset(), px(0.));
        assert_eq!(
            CaretStyle::new().with_width(px(1.)).boundary_offset(),
            px(0.)
        );
        assert_eq!(
            CaretStyle::new().with_width(px(2.)).boundary_offset(),
            px(1.)
        );
    }

    #[gpui::test]
    fn a_font_measured_caret_spans_both_sides_of_the_baseline(cx: &mut gpui::TestAppContext) {
        struct Empty;
        impl gpui::Render for Empty {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                gpui::div()
            }
        }

        let (_, cx) = cx.add_window_view(|_, _| Empty);
        cx.update(|window, _| {
            let size = px(16.);
            let text_system = window.text_system();
            let font_id = text_system.resolve_font(&window.text_style().font());

            assert!(
                font_caret_height(size, window) > text_system.ascent(font_id, size),
                "GPUI reports a font's descent negative, so a caret no taller than the \
                 ascent means the sign was added rather than subtracted"
            );
        });
    }

    #[test]
    fn the_radius_never_outgrows_the_caret_it_rounds() {
        let style = CaretStyle::new().with_width(px(2.)).with_radius(px(6.));

        assert_eq!(style.clamped_radius(size(px(2.), px(16.))), px(1.));
        assert_eq!(
            style.clamped_radius(size(px(2.), px(1.))),
            px(0.5),
            "a caret shorter than it is wide rounds by its height"
        );
    }
}
