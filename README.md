# exabind

A TUI for viewing KDE shortcuts with an interactive keyboard layout and animated visualizations.

This can be seen as a "tech demo" for [tachyonfx](https://github.com/junkdog/tachyonfx). 

Feel free to open issues for any features requests or bugs you find!

![exabind](demo/exabind-02.gif)

## Features

- Interactive keyboard layout visualization with LED effects and key highlighting
- Parse and display shortcuts from:
    - KDE global shortcuts
    - ~~JetBrains IDE keymap files~~
- Filter shortcuts by modifier keys (Ctrl, Alt, Shift, Meta)
- Categorized shortcut display with animated transitions
- Beautiful TUI powered by [ratatui](https://github.com/ratatui-org/ratatui)
- [Catppuccin](https://github.com/catppuccin/catppuccin) color scheme

## Running

```bash
cargo run --release
```

## Usage

```bash
# View KDE global shortcuts (~/.config/kglobalshortcutsrc")
exabind 

# or specify a custom path
exabind path/to/kglobalshortcutsrc
```

### Controls

| Key                         | Action                      |
|-----------------------------|-----------------------------|
| `q`                         | Quit                        |
| `↑/↓`                       | Navigate categories         |
| `Esc`                       | Deselect category           |
| `Ctrl`/`Alt`/`Shift`/`Meta` | Toggle modifier key filters |
