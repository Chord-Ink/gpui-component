use gpui::{Context, Task};

use crate::{CaretMotion, Theme};

/// To manage the Input cursor blinking.
///
/// The timing comes from [`CaretMotion`] on the Base theme.
///
/// Every loop will notify the view to update the `visible`, and Input will observe this update to touch repaint.
///
/// The input painter will check if this in visible state, then it will draw the cursor.
pub(crate) struct BlinkCursor {
    visible: bool,
    paused: bool,
    epoch: usize,

    _task: Task<()>,
}

impl BlinkCursor {
    pub(crate) fn new() -> Self {
        Self {
            visible: false,
            paused: false,
            epoch: 0,
            _task: Task::ready(()),
        }
    }

    fn motion(cx: &Context<Self>) -> CaretMotion {
        Theme::global(cx).caret_motion()
    }

    /// Show the caret and start blinking it. Focus lands here, so the caret
    /// appears now rather than on the next tick.
    pub(crate) fn start(&mut self, cx: &mut Context<Self>) {
        self.paused = false;
        self.visible = true;
        cx.notify();
        let epoch = self.next_epoch();
        self.schedule(epoch, cx);
    }

    /// Hide the caret and drop the timer. Leaves no state behind, so a pause in
    /// flight cannot resume a blurred field.
    pub(crate) fn stop(&mut self, cx: &mut Context<Self>) {
        self.paused = false;
        self.visible = false;
        self.epoch = 0;
        self._task = Task::ready(());
        cx.notify();
    }

    fn next_epoch(&mut self) -> usize {
        self.epoch += 1;
        self.epoch
    }

    /// Wait one interval, then blink. A steady caret keeps an idle timer, so
    /// turning blinking back on reaches an already-focused field.
    fn schedule(&mut self, epoch: usize, cx: &mut Context<Self>) {
        let interval = Self::motion(cx).interval();
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(interval).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.blink(epoch, cx));
            }
        });
    }

    fn blink(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if self.paused || epoch != self.epoch {
            self.visible = true;
            return;
        }

        if Self::motion(cx).is_blinking() {
            self.visible = !self.visible;
            cx.notify();
        } else if !self.visible {
            self.visible = true;
            cx.notify();
        }

        let epoch = self.next_epoch();
        self.schedule(epoch, cx);
    }

    pub(crate) fn visible(&self) -> bool {
        // Keep showing the cursor if paused
        self.paused || self.visible
    }

    /// Pause the blinking, and hold the caret visible for [`CaretMotion::pause`]
    /// before it resumes.
    pub(crate) fn pause(&mut self, cx: &mut Context<Self>) {
        self.paused = true;
        self.visible = true;
        cx.notify();

        let epoch = self.next_epoch();
        let pause = Self::motion(cx).pause();
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(pause).await;

            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.paused = false;
                    this.blink(epoch, cx);
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, Entity, TestAppContext};
    use std::time::Duration;

    const INTERVAL: Duration = Duration::from_millis(500);
    const PAUSE: Duration = Duration::from_millis(500);

    fn new_cursor(cx: &mut TestAppContext) -> Entity<BlinkCursor> {
        cx.update(|cx| cx.new(|_| BlinkCursor::new()))
    }

    fn is_visible(cursor: &Entity<BlinkCursor>, cx: &mut TestAppContext) -> bool {
        cursor.read_with(cx, |cursor, _| cursor.visible())
    }

    fn tick(cx: &mut TestAppContext, duration: Duration) {
        cx.executor().advance_clock(duration);
        cx.run_until_parked();
    }

    fn set_motion(cx: &mut TestAppContext, motion: CaretMotion) {
        cx.update(|cx| Theme::update(cx, |theme| theme.with_caret_motion(motion)));
    }

    #[gpui::test]
    fn a_focused_caret_shows_at_once_and_then_alternates(cx: &mut TestAppContext) {
        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        assert!(
            is_visible(&cursor, cx),
            "focusing a field has to show the caret now, not on the next tick"
        );

        tick(cx, INTERVAL);
        assert!(!is_visible(&cursor, cx));
        tick(cx, INTERVAL);
        assert!(is_visible(&cursor, cx));
    }

    #[gpui::test]
    fn starting_shows_the_caret_however_the_last_blink_left_it(cx: &mut TestAppContext) {
        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        tick(cx, INTERVAL);
        assert!(!is_visible(&cursor, cx));
        cursor.update(cx, |cursor, cx| cursor.stop(cx));
        cursor.update(cx, |cursor, cx| cursor.start(cx));
        assert!(
            is_visible(&cursor, cx),
            "clicking back into a field must not leave it caret-less for an interval"
        );

        // Reactivating the window starts a field that never stopped.
        cursor.update(cx, |cursor, cx| cursor.start(cx));
        assert!(
            is_visible(&cursor, cx),
            "starting an already-visible caret must leave it visible"
        );
    }

    #[gpui::test]
    fn a_blur_during_the_typing_pause_does_not_strand_the_caret(cx: &mut TestAppContext) {
        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        // Type, blur and refocus, all inside the pause window.
        cursor.update(cx, |cursor, cx| cursor.pause(cx));
        cursor.update(cx, |cursor, cx| cursor.stop(cx));
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        tick(cx, PAUSE);
        assert!(
            !is_visible(&cursor, cx),
            "the caret is still blinking after a blur inside the typing pause"
        );
    }

    #[gpui::test]
    fn typing_holds_the_caret_visible_for_the_pause(cx: &mut TestAppContext) {
        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        tick(cx, INTERVAL);
        assert!(!is_visible(&cursor, cx));

        cursor.update(cx, |cursor, cx| cursor.pause(cx));
        assert!(is_visible(&cursor, cx), "a keystroke shows the caret");

        tick(cx, PAUSE / 2);
        assert!(is_visible(&cursor, cx), "and holds it for the pause");

        tick(cx, PAUSE);
        assert!(!is_visible(&cursor, cx), "after which it blinks again");
    }

    #[gpui::test]
    fn a_steady_caret_stays_visible(cx: &mut TestAppContext) {
        set_motion(cx, CaretMotion::new().with_blinking(false));

        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));

        for _ in 0..4 {
            tick(cx, INTERVAL);
            assert!(is_visible(&cursor, cx), "a steady caret never turns off");
        }
    }

    #[gpui::test]
    fn blinking_can_be_turned_back_on_under_a_focused_field(cx: &mut TestAppContext) {
        set_motion(cx, CaretMotion::new().with_blinking(false));

        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));
        tick(cx, INTERVAL);
        assert!(is_visible(&cursor, cx));

        set_motion(cx, CaretMotion::new());
        tick(cx, INTERVAL);

        assert!(
            !is_visible(&cursor, cx),
            "a field already focused when the setting changed has to pick it up too"
        );
    }

    #[gpui::test]
    fn a_blurred_caret_is_hidden_and_costs_no_timer(cx: &mut TestAppContext) {
        let cursor = new_cursor(cx);
        cursor.update(cx, |cursor, cx| cursor.start(cx));
        cursor.update(cx, |cursor, cx| cursor.stop(cx));

        assert!(!is_visible(&cursor, cx));
        tick(cx, INTERVAL * 4);
        assert!(
            !is_visible(&cursor, cx),
            "nothing may resurrect the caret of a field that lost focus"
        );
    }
}
