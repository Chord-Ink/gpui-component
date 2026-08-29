---
order: -4
---

# Theme

所有组件都支持内置主题系统。[ActiveTheme] trait 用于访问当前主题中的颜色值：

```rs
use gpui_component::{ActiveTheme as _};

// Access theme colors in your components
cx.theme().primary
cx.theme().background
cx.theme().foreground
```

因此，如果你希望组件使用当前主题的颜色，组件或视图就需要运行在带有 [App] 上下文的环境中。

## 渐变背景

主题颜色值继续兼容既有的字符串格式：

```json
{
  "colors": {
    "button.primary.background": "#4F46E5"
  }
}
```

支持渐变渲染的背景 token 也可以使用 CSS 风格的两段线性渐变：

```json
{
  "colors": {
    "button.primary.background": "linear-gradient(135deg, #4F46E5, #06B6D4)",
    "button.primary.hover.background": "linear-gradient(to right, red-500 25%, blue-600 75%)"
  }
}
```

`cx.theme().button_primary` 等顶层字段仍然是纯色 `Hsla`，保持兼容。需要完整 resolved token 时使用 `cx.theme().tokens.button_primary`；其中 `.color` 是纯色代表色，`.background` 是实际配置的 `Background`，包含渐变。

## 光标（Caret）

所有文本输入框、多行文本框、编辑器和 OTP 单元格绘制的都是同一个光标，样式由主题决定。默认值跟随目标平台的约定，而不是统一的一套风格——在 macOS 上看着原生的光标，放到 Windows 上就像网页：

| | 宽度 | 端头 | 颜色 |
| --- | --- | --- | --- |
| macOS | 2px | 圆角（胶囊形） | 系统插入点颜色 |
| Windows | 1px | 直角 | `caret` |
| Linux | 1px | 直角 | `caret` |
| Web（wasm） | 1px | 直角 | `caret` |

自 Sonoma 起，macOS 用系统插入点颜色绘制 2pt 宽的胶囊形光标。Windows、GNOME、KDE 以及各家浏览器引擎都用文本颜色绘制 1px 宽的直角竖线，因此其余平台沿用这一约定。若主题的 `radius` 为 `0`，光标同样变为直角，与胶囊形元素被拉直的方式一致。

光标的高度为字体的 ascent 加 descent，并取整到整数像素——不使用行高的某个比例，因为行盒比字面本身宽松或紧凑时，比例高度就会与文本脱节。空输入框直接测量字体，因此按下第一个键时光标高度不会跳变。

### 颜色

主题一旦指定 `caret` 颜色，该颜色在所有平台生效：

```json
{
  "colors": {
    "caret": "#4F46E5"
  }
}
```

未指定时，光标交由平台决定：macOS 14 及以上使用 AppKit 绘制自身插入点的颜色，其余平台使用 `foreground`。请通过 `cx.theme().caret_color()` 读取最终颜色，而不是仅表示 token 的 `cx.theme().caret`。

该颜色跟随「系统设置 > 外观 > 突出显示颜色（Highlight color）」，而非强调色（Accent color）。macOS 出厂时突出显示颜色设为「强调颜色」，因此两者默认一致；一旦用户单独设置了突出显示颜色，光标就会跟随它——原生文本框正是如此。

这由主题决定，而非应用：指定了 `caret` 的主题在所有平台使用该颜色，未指定的则交由平台决定。若希望在 macOS 上固定光标颜色，请在主题中指定 `caret`。

### 选区（Selection）

选中文本的色相与光标一致——在 macOS 上，两者本就是同一项设置的两个色调层级：

| | 选区背景 |
| --- | --- |
| macOS | 系统突出显示颜色，30% 透明度 |
| Windows | 系统强调色，30% 透明度 |
| Linux | 系统强调色，30% 透明度 |
| Web（wasm） | `selection` |

Windows 用强调色绘制文本选区（WinUI 的 `TextControlSelectionHighlightColor`），GNOME 同样使用 30% 透明度的强调色，因此各平台都落在自己的约定上。Web 没有可读取的系统颜色，沿用主题值。

与 `caret` 一样，主题一旦指定 `selection.background`，该颜色在所有平台生效：

```json
{
  "colors": {
    "selection.background": "#3b82f680"
  }
}
```

透明度由本库决定，而非平台。AppKit 提供的是一层淡色调，用于在字形后方以不透明方式填充；而覆盖在语法高亮文本之上的淡色蒙层需要的是饱和色。

**选中文本默认保持自身颜色。** Windows、GTK3 和 Qt 都会把它改为白色，但它们均搭配**不透明**填充——改色正是为了保证文本在不透明底色上依然可读。在 30% 的蒙层下，改色只会抹平其覆盖的语法高亮颜色；这也是 Firefox 在选区背景 alpha 低于 155 时跳过改色、以及 VS Code 在明暗两套主题中都不设置 `editor.selectionForeground` 的原因。

若主题确实需要这种改色（并自行搭配不透明的 `selection.background`），指定颜色即可：

```json
{
  "colors": {
    "selection.background": "#0078d7",
    "selection.foreground": "#ffffff"
  }
}
```

它是最顶层的高亮，优先级高于语法高亮、语义高亮和应用自定义样式；但它只改变字形颜色，其下的诊断波浪线仍会标示文本。

窗口失去活动状态时，选区会淡化到一半强度而不是消失，这样切回窗口时仍能看到此前选中的内容。

### 闪烁

闪烁由主题上的 [CaretMotion] 控制，一次设置对所有输入框生效：

```rs
use std::time::Duration;
use gpui_component::{CaretMotion, Theme};

// 让光标保持常亮。
Theme::global_mut(cx).caret_motion = CaretMotion::new().with_blinking(false);

// 或者放慢闪烁。
Theme::global_mut(cx).caret_motion = CaretMotion::new()
    .with_interval(Duration::from_millis(700))
    .with_pause(Duration::from_millis(500));

Theme::sync_base(cx);
```

`interval` 是光标在显示与隐藏两个阶段各自停留的时长，`pause` 是按键后光标保持可见、随后才恢复闪烁的时长——足够长，连续输入时不会在词中途开始闪烁。常亮的光标不会启动任何定时器。

[CaretMotion]: https://docs.rs/gpui-component/latest/gpui_component/struct.CaretMotion.html

## Theme Registry

仓库在 [themes](https://github.com/longbridge/gpui-component/tree/main/themes) 目录下内置了 20+ 主题。

你可以通过 [ThemeRegistry] 来加载和监听这些主题文件：

从 registry 查找主题时使用 `themes` 数组中条目的 `name`，例如 `Ayu Light`。

```rs
use std::path::PathBuf;
use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};

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
