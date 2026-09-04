---
title: Theme
description: Customize colors, typography, radii, and light or dark appearance with the GPUI Component theme system.
order: -4
---

# Theme

All components support theming through the built-in Theme system, the [ActiveTheme] trait provides access to the current theme colors:

```rs
use gpui_kit::component::{ActiveTheme as _};

// Access theme colors in your components
cx.theme().primary
cx.theme().background
cx.theme().foreground
```

So if you want use the colors from the current theme, you should keep your component or view have [App] context.

## Gradient Backgrounds

Theme color values remain backward compatible with the existing string format:

```json
{
  "colors": {
    "button.primary.background": "#4F46E5"
  }
}
```

Background tokens that opt in to gradient rendering can also use CSS-style two-stop linear gradients:

```json
{
  "colors": {
    "button.primary.background": "linear-gradient(135deg, #4F46E5, #06B6D4)",
    "button.primary.hover.background": "linear-gradient(to right, red-500 25%, blue-600 75%)"
  }
}
```

Top-level theme fields, such as `cx.theme().button_primary`, remain solid `Hsla` values for compatibility. Code that needs the full resolved token can use `cx.theme().tokens.button_primary`; its `.color` field is the solid representative color, and its `.background` field contains the configured `Background`, including gradients.

## Caret

Every text field, textarea, editor and OTP cell draws the same caret, and the
theme decides how. The defaults follow the platform the app is built for
rather than one house style, because a caret that reads as native on macOS
reads as a web page on Windows:

| | Width | Caps | Color |
| --- | --- | --- | --- |
| macOS | 2px | rounded (a capsule) | system insertion-point color |
| Windows | 1px | square | `caret` |
| Linux | 1px | square | `caret` |
| Web (wasm) | 1px | square | `caret` |

macOS has drawn a two-point capsule caret in the system insertion-point color
since Sonoma. Windows, GNOME, KDE and every browser engine draw a square
one-pixel bar in the text color, so that is what the other targets get. A theme
whose `radius` is `0` squares the caret too, the same way it squares a pill.

The caret is as tall as its font's ascent plus descent, rounded to a whole
pixel — never a fraction of the line height, which drifts away from the text
whenever the line box is looser or tighter than the face. An empty field
measures the font directly, so the caret does not change height on the first
keystroke.

### Color

A theme that names a `caret` color owns it on every platform:

```json
{
  "colors": {
    "caret": "#4F46E5"
  }
}
```

A theme that names none leaves the caret to the platform: macOS 14 and later
answers with the color AppKit gives its own insertion point, and every other
target with `foreground`. Read the resolved value through
`cx.theme().caret_color()` rather than `cx.theme().caret`, which is only the token.

That color follows System Settings > Appearance > **Highlight color**, not the
accent color. macOS ships Highlight set to "Accent color", so the two agree
until the user picks a Highlight of their own — set them apart and the caret
follows Highlight, the way every native text field does.

This is decided by the theme, not by the app: a theme that names `caret` gets
its color everywhere, and one that does not leaves the caret to the platform.
To pin a caret color on macOS, name it in the theme.

### Selection

Selected text takes the same hue, because on macOS the caret and the selection
are one setting at two tint levels:

| | Selection background |
| --- | --- |
| macOS | system text-highlight color at 30% |
| Windows | system accent color at 30% |
| Linux | system accent color at 30% |
| Web (wasm) | `selection` |

Windows selects text with the accent (WinUI's `TextControlSelectionHighlightColor`)
and GNOME with the accent at 30%, so each platform lands on its own convention.
The web publishes no readable system color, so it keeps the theme's.

A theme that names `selection.background` owns it on every platform, exactly as
with `caret`:

```json
{
  "colors": {
    "selection.background": "#3b82f680"
  }
}
```

The alpha is this library's, not the platform's. AppKit hands out a pale tint
meant to be filled opaque behind the glyphs; a wash laid over syntax-highlighted
text needs the saturated form instead.

**Selected text keeps its own color by default.** Windows, GTK3 and Qt all
recolor it to white, but each pairs that with an *opaque* fill, where the flip is
what keeps the text readable. Under a 30% wash it would only flatten the syntax
colors it covers — which is why Firefox skips the flip whenever the selection
background's alpha is below 155, and why VS Code leaves `editor.selectionForeground`
unset in both light and dark.

A theme that wants the flip anyway — pairing it with an opaque
`selection.background` of its own — names the color:

```json
{
  "colors": {
    "selection.background": "#0078d7",
    "selection.foreground": "#ffffff"
  }
}
```

It is the topmost highlight, so it wins over syntax, semantic and application
styles alike, but it sets only the glyph color: a diagnostic underline beneath
it still marks the text.

While the window is not the active one the selection dims to half strength
rather than disappearing, so returning to a window still shows what was
selected.

### Blinking

Blinking is [CaretMotion] on the theme, and it applies to every field at once:

```rs
use std::time::Duration;
use gpui_component::{CaretMotion, Theme};

// Hold the caret steady.
Theme::global_mut(cx).caret_motion = CaretMotion::new().with_blinking(false);

// Or slow it down.
Theme::global_mut(cx).caret_motion = CaretMotion::new()
    .with_interval(Duration::from_millis(700))
    .with_pause(Duration::from_millis(500));

Theme::sync_base(cx);
```

`interval` is how long the caret rests in each of its two phases, and `pause`
is how long it is held visible after a keystroke before it resumes — long
enough that a burst of typing never starts a blink mid-word. A steady caret
schedules no timer at all.

[CaretMotion]: https://docs.rs/gpui-component/latest/gpui_component/struct.CaretMotion.html

## Theme Registry

There have more than 20 built-in themes available in [themes](https://github.com/longbridge/gpui-kit/tree/main/themes) folder.

https://github.com/longbridge/gpui-kit/tree/main/themes

And we have a [ThemeRegistry] to help us to load themes.

Use the `name` of an entry in the `themes` array, such as `Ayu Light`, when looking up a theme from the registry.

```rs
use std::path::PathBuf;
use gpui_kit::{App, SharedString};
use gpui_kit::component::{Theme, ThemeRegistry};

pub fn init(cx: &mut App) {
    let theme_name = SharedString::from("Ayu Light");
    // Load and watch themes from ./themes directory
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&theme_name)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        tracing::error!("Failed to watch themes directory: {}", err);
    }
}
```

[ActiveTheme]: https://docs.rs/gpui-component/latest/gpui_component/theme/trait.ActiveTheme.html
[ThemeRegistry]: https://docs.rs/gpui-component/latest/gpui_component/theme/struct.ThemeRegistry.html
[App]: https://docs.rs/gpui/latest/gpui/struct.App.html
