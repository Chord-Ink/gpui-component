use std::time::Duration;

use gpui::{Half as _, Hsla, Pixels, Size, Window, px};

/// How the text insertion caret is drawn.
///
/// Base paints the caret and measures horizontal scroll against it, so the
/// width is geometry as much as appearance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretStyle {
    width: Pixels,
    radius: Pixels,
    color: Hsla,
}

impl Default for CaretStyle {
    fn default() -> Self {
        Self {
            // One pixel: what Blink, Gecko, Windows, GTK and Qt all draw.
            width: px(1.),
            // Square; a rounded cap is the platform's choice.
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
    /// clamp measure against it, so this is geometry as well as color.
    pub fn with_width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// The radius of both caps, clamped at paint to half the caret's shorter
    /// side. Zero is a square caret.
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

    /// How far left of the character boundary the caret is drawn. A caret wider
    /// than a pixel straddles the boundary instead of covering the next glyph.
    pub fn boundary_offset(&self) -> Pixels {
        if self.width > px(1.) {
            self.width.half()
        } else {
            px(0.)
        }
    }

    /// The radius the caret can carry inside `size`, since a quad painted with
    /// an oversized radius lets its corners overrun each other.
    pub fn clamped_radius(&self, size: Size<Pixels>) -> Pixels {
        self.radius.min(size.width.half()).min(size.height.half())
    }
}

/// Ascent plus descent of the window's current text face at `font_size`.
///
/// GPUI reports `descent` negative below the baseline, so subtracting it is
/// what matches a caret measured from shaped text.
pub fn font_caret_height(font_size: Pixels, window: &Window) -> Pixels {
    let text_system = window.text_system();
    let font_id = text_system.resolve_font(&window.text_style().font());

    text_system.ascent(font_id, font_size) - text_system.descent(font_id, font_size)
}

/// The shortest blink a caret is allowed, one frame at 60Hz.
const MIN_INTERVAL: Duration = Duration::from_millis(16);

/// How the text insertion caret blinks.
///
/// Base owns the timer, because the blink is bound to focus, typing and window
/// activation.
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
            // Half a second in each phase, where Blink, Gecko and WinUI land.
            interval: Duration::from_millis(500),
            // One full phase, so a burst of keystrokes never blinks mid-word.
            pause: Duration::from_millis(500),
        }
    }
}

impl CaretMotion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the caret blinks at all. A steady caret still shows and hides
    /// with focus.
    pub fn with_blinking(mut self, blinking: bool) -> Self {
        self.blinking = blinking;
        self
    }

    /// How long the caret rests in each of its two phases, floored at one
    /// frame so the timer always has something to wait for.
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
