# Cards

A beautiful, fast, and minimal note-taking application built with Rust and [Iced](https://iced.rs/).

[![Rust - Cargo build](https://github.com/Azazel55605/Cards/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/Azazel55605/Cards/actions/workflows/rust.yml)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

## ✨ Overview

Cards is a modern, infinite canvas note-taking application that lets you organize your thoughts spatially. Create, arrange, and connect your ideas with beautiful cards on an interactive dot grid background.

### Key Features

- 🎨 **Infinite Canvas** - Unlimited space to organize your notes
- 📋 **Multiple Boards** - Organize cards across separate workspaces with independent grids
- 📝 **Markdown Support** - Full markdown rendering with syntax highlighting for code blocks
- 🎯 **Smart Card System** - Resizable, color-coded cards with custom icons
- ⚡ **Lightning Fast** - Built with Rust for maximum performance
- 🌓 **Dark/Light Themes** - Beautiful themes that adapt to your preference
- 🎨 **Customizable Cards** - Choose from 80+ Bootstrap icons and multiple colors
- 🔤 **Rich Text Editing** - Full markdown support including:
  - Headers (H1-H6)
  - **Bold**, *italic*, and ~~strikethrough~~
  - Code blocks with syntax highlighting (40+ languages)
  - Bulleted lists
  - Interactive checkboxes `[ ]` and `[x]`
  - Inline code
- 💾 **Auto-Save** - Your work is automatically saved per board
- 🎯 **Smooth Animations** - Fluid card movements, board transitions, and interactions
- ⌨️ **Keyboard-First** - Comprehensive keyboard shortcuts for everything
- 🎨 **Custom Text Editor** - Monospace font support with proper cursor tracking

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
| `Ctrl + Tab` | Switch to next board |
| `Ctrl + Shift + Tab` | Switch to previous board |
| `Middle Mouse Button` | Recenter canvas (animated) |
| `Esc` | Exit editing mode / Close menus |

### Text Editing

| Shortcut | Action |
|----------|--------|
| `Tab` | Insert 4 spaces |
| `Enter` | New line (or auto-complete markdown tags) |
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

### Managing Boards

Boards allow you to organize cards into separate workspaces, each with its own independent infinite canvas.

#### Creating Boards

1. Look at the **sidebar** (left side)
2. Click **"+ Add New Board"** button
3. A new board is created with a default name (Board 2, Board 3, etc.)
4. The button can be positioned at the top or bottom (toggle in Settings)

#### Switching Between Boards

- **Click** on a board name in the sidebar to switch
- **Keyboard**: Press `Ctrl + Tab` to cycle forward through boards
- **Keyboard**: Press `Ctrl + Shift + Tab` to cycle backward through boards
- Each board maintains its own cards independently

#### Renaming Boards

1. **Click twice** on a board name (or click once on the active board)
2. A text input will appear
3. Type the new name
4. Press `Enter` to save or `Esc` to cancel
5. Click outside the input to save automatically

#### Deleting Boards

1. **Hover** over a board in the sidebar
2. A **red delete button** (🗑️) appears on the right
3. **Click** the delete button
4. The board and all its cards are removed (with smooth animation)
5. Note: You cannot delete the last remaining board

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
   - **80+ Bootstrap icons** - Organized in a scrollable grid (6 per row)
   - **Multiple colors** - For visual categorization
4. The selected icon and color are immediately applied

### Interactive Checkboxes

Checkboxes in markdown are fully interactive:

1. Write checkboxes in markdown: `- [ ]` or `- [x]`
2. **Click** on the checkbox in the rendered view to toggle it
3. The checkbox state updates in real-time
4. Works seamlessly with markdown rendering

### Using the Toolbar

When a card is selected, a toolbar appears above it with formatting options:

| Button | Markdown | Description |
|--------|----------|-------------|
| `#` | `# Text` | Convert to heading |
| `B` | `**Text**` | Bold text (wraps selection) |
| `I` | `*Text*` | Italic text (wraps selection) |
| `S` | `~~Text~~` | Strikethrough (wraps selection) |
| `` ` `` | `` `Code` `` | Inline code (wraps selection) |
| `</>` | ` ```Code``` ` | Code block (wraps selection) |
| `•` | `- Item` | Bullet point |
| 📋 | - | Duplicate card (creates copy at offset) |
| 🗑️ | - | Delete card (removes from canvas) |

### Resizing Cards

1. **Hover over** or **select** a card
2. A **resize handle** (↘) appears in the bottom-right corner
3. **Click and drag** the handle to resize the card
4. Cards have a **minimum size** (200x150) and can grow infinitely
5. Resizing is **grid-snapped** for perfect alignment
6. **Smooth animation** transitions the card to its new size
7. Mouse cursor changes to resize indicator when hovering the handle

### Moving Cards

1. **Click and hold** on a card's header (colored bar at the top)
2. **Drag** the card to your desired position
3. The card will snap to the grid when you release
4. Cards can be moved while **selected** or **unselected**
5. **Selected cards** can still be dragged normally

### Canvas Navigation

- **Click and drag** on empty space to pan the canvas
- **Middle-mouse click** to instantly recenter to origin (with animation)
- **Scroll** to navigate vertically and horizontally
- **Recenter the view**: Press `Ctrl+0` to smoothly return to origin (0, 0)
- Canvas has **infinite space** in all directions
- Smooth **scrolling animations** for better user experience

### Theme & Settings

1. Click the **sidebar toggle** button (left edge of the screen)
2. The sidebar slides in with smooth animation
3. Access:
   - **Theme switcher** (Light/Dark/Auto) with instant preview
   - **Settings panel** (⚙️ gear icon) for detailed customization
   - **Board management** - Create, rename, delete, and switch boards
   - Animation preferences (enable/disable)
   - Font selection (multiple monospace fonts)
   - Font size adjustment (14-24pt)
   - New board button position (top/bottom)

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

Configuration is automatically stored in:
- **Linux**: `~/.config/cards/config.toml`
- **macOS**: `~/Library/Application Support/cards/config.toml`
- **Windows**: `%APPDATA%\cards\config.toml`

### Auto-Healing Configuration

The configuration system is self-healing:
- Missing settings are automatically added with default values
- Outdated settings are automatically removed
- No manual config file editing needed
- Safe to delete - will regenerate with defaults

### Available Settings

**General**
- Theme (Light/Dark/Auto)
- Enable/disable animations
- New board button position (top/bottom)

**Appearance**
- Font family (multiple monospace fonts)
- Font size (14-24pt)

All settings can be changed through the in-app Settings panel.

## 📝 Tips & Tricks

1. **Quick Markdown**: Type `<md>` and press `>` - the closing tag auto-completes with the cursor positioned perfectly
2. **Multi-line Code**: Use triple backticks with a language name for syntax-highlighted code blocks
3. **Organization**: Use different colors and icons to categorize your cards
4. **Multiple Boards**: Organize related cards into separate boards - great for different projects
5. **Keyboard Navigation**: Press `Ctrl+Tab` to quickly switch between boards without touching the mouse
6. **Quick Rename**: Double-click or click twice on any board name to rename it instantly
7. **Interactive Todos**: Checkboxes in markdown are clickable - perfect for task lists
8. **Card Duplication**: Use the duplicate button (📋) to quickly create similar cards
9. **Smooth Workflow**: Cards auto-save per board, so you never lose your work
10. **Font Customization**: Choose your preferred monospace font and size in settings
11. **Grid Snapping**: Cards and resize operations snap to the grid for perfect alignment
12. **Toolbar Shortcuts**: Select text and use toolbar buttons to wrap it in markdown formatting
13. **Hover Actions**: Hover over boards to reveal the delete button
14. **Canvas Reset**: Lost your cards off-screen? Press `Ctrl+0` or middle-click to recenter

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🙏 Acknowledgments

- Built with the amazing [Iced](https://iced.rs/) framework
- Icons provided by [Bootstrap Icons](https://icons.getbootstrap.com/)

## 📧 Contact

For questions or feedback, please open an issue on GitHub.

---

**Made with ❤️ and Rust**

