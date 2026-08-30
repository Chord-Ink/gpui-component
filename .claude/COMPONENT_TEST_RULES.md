# GPUI Component Testing Rules

Do not add a test unless it pins logic that could silently break. A component
with no logic gets no test. Coverage is not a goal.

## Write a test for

- Algorithms and data structures: layout trees, rope operations, text wrapping,
  mask patterns, undo and history, index paths, selection math, geometry,
  placement math, plot scales and shapes, virtual-list ranges, color conversion,
  parsing, fuzzy matching, calendar math.
- Interaction driven through `TestAppContext` / `VisualTestContext`: focus order
  and tab stops, keyboard versus pointer activation, disabled blocking and
  propagation, IME and undo transactions, scroll and drag state transitions.
- Round-trips of persisted state.
- A specific edge case, boundary, or off-by-one that a bug exposed.

Use plain `#[test]` for logic that needs no app context. Reach for
`#[gpui::test]` only when the behavior needs a window or event dispatch.

## Never write

```rust
// Builder chain that asserts back what it just set.
let button = Button::new("b").primary().large().disabled(false);
assert_eq!(button.variant, ButtonVariant::Primary);

// Proof that a path, type, or method still exists.
let _: fn(&App) -> ButtonCustomVariant = ButtonCustomVariant::new;
fn takes(_: Button) {}

// Presentation constants.
assert_eq!(button.padding(), px(8.));

// A one-line enum helper.
assert!(ButtonVariant::Link.is_link());
```

Also skip: getter/setter round-trips, `Default::default()` field checks, tests
that restate std or GPUI behavior, near-identical tests differing only by a
size or variant, and smoke tests that render and assert nothing.

For pure visual or sizing changes, add no test at all.

## Shape

Keep tests in one `#[cfg(test)] mod tests` at the end of the file. Combine
related cases into a single function rather than splitting one assertion per
test. Name the function after the behavior it pins, not after the method it
calls: `disabled_button_blocks_parent_click`, not `test_button_disabled`.
