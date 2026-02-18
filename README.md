# Cards

A beautiful, fast, and minimal note-taking application built with Rust and [Iced](https://iced.rs/).

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

## ✨ Overview

Cards is a modern, infinite canvas note-taking application that lets you organize your thoughts spatially. Create, arrange, and connect your ideas with beautiful cards on an interactive dot grid background.

### Key Features

- 🎨 **Infinite Canvas** - Unlimited space to organize your notes
- 📝 **Markdown Support** - Full markdown rendering with syntax highlighting for code blocks
- 🎯 **Smart Card System** - Resizable, color-coded cards with custom icons
- ⚡ **Lightning Fast** - Built with Rust for maximum performance
- 🌓 **Dark/Light Themes** - Beautiful themes that adapt to your preference
- 🎨 **Customizable Cards** - Choose from 80+ icons and multiple colors
- 🔤 **Rich Text Editing** - Full markdown support including:
  - Headers (H1-H6)
  - **Bold**, *italic*, and ~~strikethrough~~
  - Code blocks with syntax highlighting (40+ languages)
  - Bulleted lists
  - Checkboxes `[ ]` and `[x]`
  - Inline code
- 💾 **Auto-Save** - Your work is automatically saved
- 🎯 **Smooth Animations** - Fluid card movements and interactions

## 🖼️ Screenshots

> *Add your screenshots here*

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/Cards.git
cd Cards

# Build the release version
cargo build --release

# Run the application
cargo run --release
```

## ⌨️ Keyboard Shortcuts

### Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + A` | Select all text |
| `Ctrl + C` | Copy selected text |
| `Ctrl + X` | Cut selected text |
| `Ctrl + V` | Paste text |
| `Ctrl + 0` | Recenter canvas to origin |
| `Esc` | Exit editing mode / Close menus |

### Text Editing

| Shortcut | Action |
|----------|--------|
| `Tab` | Insert 4 spaces |
| `Enter` | New line |
| `Backspace` | Delete previous character |
| `Ctrl + Backspace` | Delete previous word |
| `Delete` | Delete next character |
| `Ctrl + Delete` | Delete next word |

### Navigation

| Shortcut | Action |
|----------|--------|
| `Arrow Keys` | Move cursor |
| `Ctrl + Arrow Left/Right` | Jump to previous/next word |
| `Home` | Move to start of line |
| `End` | Move to end of line |
| `Ctrl + Home` | Move to start of document |
| `Ctrl + End` | Move to end of document |

### Text Selection

| Shortcut | Action |
|----------|--------|
| `Shift + Arrow Keys` | Select text |
| `Shift + Ctrl + Arrow Left/Right` | Select word by word |

## 📖 How to Use

### Creating Cards

1. **Right-click** anywhere on the canvas to open the context menu
2. Click **"Add Card"** to create a new card at that position
3. The card will appear and automatically enter edit mode

### Editing Cards

1. **Click once** on a card to select it
2. **Double-click** on a card to start editing
3. Type your content directly
4. Click outside the card or press `Esc` to finish editing

### Using Markdown

Cards supports rich markdown formatting. To use markdown features:

1. Wrap your markdown content in `<md>` tags
2. The app will auto-complete when you type `<md>` + `>`
3. Write your markdown between the tags

**Example:**

```
<md>
# This is a heading

This is **bold** and this is *italic*.

```python
def hello():
    print("Code with syntax highlighting!")
```

- Bullet point
- [x] Checked item
- [ ] Unchecked item
</md>
```

### Customizing Cards

1. **Select a card** by clicking on it
2. Click the **colored circle** in the top-left of the card
3. Choose from:
   - **80+ icons** from Bootstrap Icons
   - **Multiple colors** for visual organization

### Using the Toolbar

When a card is selected, a toolbar appears above it with formatting options:

| Button | Markdown | Description |
|--------|----------|-------------|
| `#` | `# Text` | Convert to heading |
| `B` | `**Text**` | Bold text |
| `I` | `*Text*` | Italic text |
| `S` | `~~Text~~` | Strikethrough |
| `` ` `` | `` `Code` `` | Inline code |
| `</>` | ` ```Code``` ` | Code block |
| `•` | `- Item` | Bullet point |
| 📋 | - | Duplicate card |
| 🗑️ | - | Delete card |

### Resizing Cards

1. **Hover over** or **select** a card
2. A **resize handle** appears in the bottom-right corner
3. **Click and drag** the handle to resize the card
4. Cards snap to the grid for perfect alignment

### Moving Cards

1. **Click and hold** on a card's header (colored bar at the top)
2. **Drag** the card to your desired position
3. The card will snap to the grid when you release

### Canvas Navigation

- **Click and drag** on empty space to pan the canvas
- **Middle-mouse drag** for quick panning
- **Scroll** to navigate vertically
- **Recenter the view**: Press `Ctrl+0` to instantly return to origin (0, 0)
- Use the **sidebar** to toggle settings and themes

### Theme & Settings

1. Click the **sidebar toggle** button (left side of the screen)
2. Access:
   - **Theme switcher** (Light/Dark/Auto)
   - **Settings panel** for customization
   - Dot grid appearance options
   - Animation preferences

## 🎨 Supported Languages for Syntax Highlighting

Code blocks support syntax highlighting for 40+ languages including:

- **Systems**: C, C++, C#, Rust, Go
- **Web**: JavaScript, TypeScript, HTML, CSS, PHP
- **Scripting**: Python, Ruby, Perl, Bash, PowerShell
- **Data**: JSON, YAML, TOML, XML
- **Database**: SQL
- **JVM**: Java, Kotlin, Scala
- **Mobile**: Swift, Objective-C
- **Functional**: Haskell, Elixir, Erlang
- And many more...

## 🏗️ Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Iced](https://iced.rs/) - Cross-platform GUI library
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) - Markdown parser
- [syntect](https://github.com/trishume/syntect) - Syntax highlighting
- [Bootstrap Icons](https://icons.getbootstrap.com/) - Icon library

## 🛠️ Configuration

Configuration is stored in:
- **Linux**: `~/.config/cards/config.toml`
- **macOS**: `~/Library/Application Support/cards/config.toml`
- **Windows**: `%APPDATA%\cards\config.toml`

## 📝 Tips & Tricks

1. **Quick Markdown**: Type `<md>` and the closing tag auto-completes with the cursor positioned perfectly
2. **Multi-line Code**: Use triple backticks with a language name for syntax-highlighted code blocks
3. **Organization**: Use different colors and icons to categorize your cards
4. **Keyboard First**: Most operations can be done without touching the mouse
5. **Smooth Workflow**: Cards auto-save, so you never lose your work

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🙏 Acknowledgments

- Built with the amazing [Iced](https://iced.rs/) framework
- Icons provided by [Bootstrap Icons](https://icons.getbootstrap.com/)

## 📧 Contact

For questions or feedback, please open an issue on GitHub.

---

**Made with ❤️ and Rust**

