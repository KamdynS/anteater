# iTerm2 Theme Support

Anteater supports iTerm2 color schemes, giving you full control over the debugger's appearance.

## Using Custom Themes

### Quick Start

The default theme is "Default Dark" (similar to base16-ocean). To change it:

1. Download an iTerm2 color scheme (`.itermcolors` file)
2. Place it in `~/.config/anteater/themes/`
3. Set `theme = "YourTheme"` in `~/.config/anteater/config.toml`

### Finding Themes

Popular iTerm2 theme collections:

- [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes) - 200+ themes
- [base16-iterm2](https://github.com/martinlindhe/base16-iterm2) - base16 color schemes
- [Dracula](https://draculatheme.com/iterm/) - The Dracula theme
- [Nord](https://www.nordtheme.com/ports/iterm2) - Nord theme
- [One Dark](https://github.com/one-dark/iterm-one-dark-theme) - Atom's One Dark

### Theme File Format

iTerm2 themes are XML property list files (`.itermcolors`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Foreground Color</key>
    <dict>
        <key>Red Component</key>
        <real>0.75294117647058822</real>
        <key>Green Component</key>
        <real>0.77254901960784317</real>
        <key>Blue Component</key>
        <real>0.80784313725490198</real>
    </dict>
    <!-- ... more colors ... -->
</dict>
</plist>
```

## How Themes Map to Anteater UI

| iTerm2 Color | Anteater Usage |
|--------------|----------------|
| Foreground Color | Text, variable names, code |
| Background Color | Panel backgrounds, window fill |
| Selection Color | Selected text, highlighted items |
| Cursor Color | Text cursor in inputs |
| Ansi 0-7 | Standard terminal colors |
| Ansi 8-15 | Bright terminal colors (syntax highlighting) |

### Specific Mappings

- **Ansi 1 (Red)**: Error messages, breakpoint dots
- **Ansi 2 (Green)**: Success indicators, "Owned" state
- **Ansi 3 (Yellow)**: Warnings, "PartiallyMoved" state
- **Ansi 4 (Blue)**: Hyperlinks, "Borrowed" state, current line
- **Ansi 5 (Magenta)**: Keywords in syntax highlighting
- **Ansi 6 (Cyan)**: String literals
- **Ansi 9 (Bright Red)**: Moved/dropped variables
- **Ansi 14 (Bright Cyan)**: Types, struct names

## Creating Your Own Theme

1. Start with an existing `.itermcolors` file or use an online generator
2. Adjust colors to your preference
3. Test in Anteater
4. Share with the community!

### Theme Guidelines

For best readability:

- **Contrast**: Ensure sufficient contrast (WCAG AA: 4.5:1 minimum)
- **Ownership states**: Make sure moved/borrowed/owned are visually distinct
- **Syntax highlighting**: Test with real Rust code to verify readability
- **Dark vs Light**: Support both if possible

## JetBrains Mono Font

Anteater uses JetBrains Mono as the default monospace font for:
- Source code
- Hex dumps
- Registers
- Memory addresses

If JetBrains Mono isn't installed on your system, egui will fall back to the default monospace font.

### Installing JetBrains Mono

**macOS:**
```bash
brew tap homebrew/cask-fonts
brew install font-jetbrains-mono
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install fonts-jetbrains-mono

# Arch
sudo pacman -S ttf-jetbrains-mono
```

**Windows:**
Download from [JetBrains website](https://www.jetbrains.com/lp/mono/)

## Future Enhancements

- [ ] Runtime theme switching (View → Change Theme)
- [ ] Theme preview panel
- [ ] Embedded JetBrains Mono font (no system install needed)
- [ ] Per-panel theme overrides
- [ ] Custom syntax highlighting themes (separate from iTerm2)
- [ ] Light theme support
- [ ] Theme hot-reloading during development

## Example Themes

### Recommended for Debugging

1. **Dracula** - High contrast, easy on eyes
2. **Nord** - Low contrast, calming
3. **One Dark** - Balanced, familiar to VSCode users
4. **Solarized Dark** - Carefully designed color relationships
5. **Gruvbox** - Warm, retro aesthetic

### Not Recommended

- Very low contrast themes (hard to read small text)
- Themes with similar colors for red/green (color accessibility)
- Extremely bright themes (eye strain during long sessions)

## Troubleshooting

**Theme not loading:**
- Check file path: `~/.config/anteater/themes/YourTheme.itermcolors`
- Verify XML is valid (use `xmllint` or similar)
- Check Anteater logs for parsing errors

**Colors look wrong:**
- Some iTerm2 themes use non-standard key names
- File an issue with the theme name and we'll add support

**Font doesn't look right:**
- Verify JetBrains Mono is installed system-wide
- Restart Anteater after font installation
- Check egui logs for font loading messages
