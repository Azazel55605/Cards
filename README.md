# Cards

A beautiful, fast, and minimal note-taking application built with Rust and [Iced](https://iced.rs/).

[![Rust - Cargo build](https://github.com/Azazel55605/Cards/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/Azazel55605/Cards/actions/workflows/rust.yml)
[![Release](https://github.com/Azazel55605/Cards/actions/workflows/release.yml/badge.svg)](https://github.com/Azazel55605/Cards/actions/workflows/release.yml)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

## ✨ Overview

Cards is a modern, infinite-canvas note-taking application that lets you organise your thoughts spatially. Create, arrange, and connect ideas with beautiful cards on an interactive dot-grid background.

### Key Features

- 🎨 **Infinite Canvas** — Unlimited space across a smooth dot grid with zoom (40%–500%) and pan
- 📋 **Multiple Boards** — Separate canvases per board with independent cards; switch instantly; drag cards between boards
- 📝 **Markdown Support** — Full markdown rendering inside `<md>` tags with 40+ language syntax highlighting
- 🎯 **Smart Cards** — Resizable, collapsible, colour-coded cards with 80+ Bootstrap icons
- 🖼️ **Image Cards** — Embed raster images (PNG/JPEG/GIF/BMP/WebP) or SVG files directly on the canvas
- 🗺️ **Minimap** — Overview overlay in the top-right corner; click or drag to pan; toggle with `M`
- ⚡ **Fast & Native** — Built entirely in Rust for maximum responsiveness
- 🌓 **Dark / Light Theme** — Smooth animated diagonal-wipe transition between themes
- 🎨 **Accent Colours** — Choose your accent colour; applied to borders, highlights, gradients, and the sidebar
- 🔤 **Rich Text Editor** — Custom monospace editor with cursor, selection, word navigation, and system clipboard
- 🔲 **Interactive Checkboxes** — Click to toggle checkboxes directly in the rendered card view
- 💾 **Per-Board Auto-Save** — Cards saved per board automatically every 30 seconds
- 🎬 **Smooth Animations** — Card move/resize, collapse, board transitions, settings open/close, theme wipe
- ⌨️ **Keyboard-First** — Full shortcut reference available in-app under Settings → Shortcuts
- 🛠️ **Self-Healing Config** — Missing or obsolete config keys are fixed automatically on startup
- ✅ **Delete Confirmation** — Optional confirmation dialog before deleting a card (toggleable in settings)

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

# Build and run
cargo run --release
```

---

## ⌨️ Keyboard Shortcuts

### Canvas

| Shortcut | Action |
|----------|--------|
| `Middle Mouse` | Pan canvas |
| `Scroll` | Pan canvas vertically / horizontally |
| `Click + Drag` | Pan canvas (on empty space) |

### Zoom

| Shortcut | Action |
|----------|--------|
| `Ctrl + Scroll` | Zoom in / out toward cursor |
| `Ctrl + +` | Zoom in (40% – 500%) |
| `Ctrl + -` | Zoom out (40% – 500%) |
| `Ctrl + 0` | Recenter canvas + reset zoom to 100% |
| Zoom bar `−` / `%` / `+` | Zoom out / reset / zoom in |

> Zoom shortcuts match the physical key position and work across keyboard layouts (QWERTY, QWERTZ, AZERTY, etc.)

### Boards

| Shortcut | Action |
|----------|--------|
| `Ctrl + Tab` | Switch to next board |
| `Ctrl + Shift + Tab` | Switch to previous board |
| `Double-click board` | Rename board inline |

### Cards

| Shortcut | Action |
|----------|--------|
| `N` | New card at mouse position (or canvas centre) |
| `Right-click canvas` | Context menu → Add Card |
| `Click` | Edit card |
| `Drag header` | Move card |
| `Drag header to board` | Move card to another board |
| `Drag ↘ handle` | Resize card |
| `▶ chevron` | Collapse / expand card |
| `Delete` | Delete selected card(s) |
| `Ctrl + D` | Duplicate selected card(s) |
| `Esc` | Stop editing / deselect |

### Multi-select

| Shortcut | Action |
|----------|--------|
| `Drag empty canvas` | Box-select cards |
| `Drag header` | Move all selected cards |
| `Delete` | Delete all selected cards |
| `Ctrl + D` | Duplicate all selected cards |

### Text Editing *(inside a card)*

| Shortcut | Action |
|----------|--------|
| `Tab` | Insert 4 spaces |
| `Enter` | New line |
| `Backspace` | Delete previous character |
| `Ctrl + Backspace` | Delete previous word |
| `Delete` | Delete next character |
| `Ctrl + Delete` | Delete next word |

### Cursor Navigation *(inside a card)*

| Shortcut | Action |
|----------|--------|
| `Arrow Keys` | Move cursor |
| `Ctrl + ← / →` | Jump to previous / next word |
| `Home` | Move to start of line |
| `End` | Move to end of line |

### Text Selection *(inside a card)*

| Shortcut | Action |
|----------|--------|
| `Shift + Arrows` | Extend selection |
| `Shift + Ctrl + ← / →` | Extend selection word by word |
| `Click + Drag` | Select text with mouse |
| `Ctrl + A` | Select all text |

### Clipboard *(inside a card)*

| Shortcut | Action |
|----------|--------|
| `Ctrl + C` | Copy selected text |
| `Ctrl + X` | Cut selected text |
| `Ctrl + V` | Paste from system clipboard |

### Toolbar *(card selected)*

| Button | Markdown | Description |
|--------|----------|-------------|
| `#` | `# Text` | Heading prefix |
| `B` | `**Text**` | Bold (wraps selection) |
| `I` | `*Text*` | Italic (wraps selection) |
| `S` | `~~Text~~` | Strikethrough (wraps selection) |
| `` ` `` | `` `Code` `` | Inline code (wraps selection) |
| `</>` | ```` ```Code``` ```` | Code block (wraps selection) |
| `•` | `- Item` | Bullet point |
| Duplicate | — | Duplicate card (inherits size) |
| Delete | — | Delete card (confirmation dialog if enabled) |

### App

| Shortcut | Action |
|----------|--------|
| `M` | Toggle minimap |
| `Esc` | Close menus / dialogs / settings |

> All shortcuts are also listed inside the app under **Settings → Shortcuts**.

---

## 📖 How to Use

### Managing Boards

Boards give you separate infinite canvases — great for different projects or contexts.

#### Creating Boards
Click the **+** board button at the top of the sidebar.

#### Switching Boards
- **Click** a board name in the sidebar, or use `Ctrl + Tab` / `Ctrl + Shift + Tab`
- Each board's cards are saved and restored independently

#### Renaming Boards
**Double-click** a board name to edit it inline. Press `Enter` to confirm or `Esc` to cancel.

#### Deleting Boards
**Hover** over a board — a red delete button appears on the right. Click it to remove the board.
You cannot delete the last remaining board.

---

### Creating Cards

- **`N`** — creates a card at the mouse position, or at viewport centre if the mouse is over the sidebar
- **Right-click** empty canvas space → **Add Card**

### Editing Cards

1. **Click** a card to start editing
2. Type your content (plain text or markdown inside `<md>` tags)
3. Click outside or press `Esc` to stop editing and render the card

### Using Markdown

Wrap content in `<md>` … `</md>` to enable markdown rendering:

```
<md>
# Heading

**Bold**, *italic*, ~~strikethrough~~, `inline code`

```python
def hello():
    print("Syntax highlighted!")
```

- Bullet item
- [x] Done
- [ ] To-do
</md>
```

**Auto-complete:** Type `<md>` followed by `>` — the closing tag and an empty line are inserted automatically with the cursor positioned inside.

### Image Cards

Switch a card's type to **Image** via the card toolbar to embed images directly on the canvas:

- Supported formats: PNG, JPEG, GIF, BMP, WebP, SVG
- Images scale to fit the card and can be resized by dragging the **↘ handle**

### Customising Cards

1. **Select** a card (single click)
2. Click the **coloured circle** in the card's top-left to open the icon/colour picker:
   - **80+ Bootstrap icons** in a scrollable 6-per-row grid, tinted in the card's colour
   - **10 accent colours** as colour circles

### Resizing Cards

Hover over or select a card — a **↘ handle** appears in the bottom-right corner. Drag it to resize. Cards snap to the dot grid and have a minimum size.

### Moving Cards

Drag the **coloured header bar** at the top of any card. Cards snap to the grid on release and can be moved while selected or unselected.

### Moving Cards Between Boards

While dragging a card's header bar, the sidebar switches to a **"Move card to…"** panel. Drag the card over the target board name (it highlights) and release to move it there.

### Collapsing Cards

Click the **▶ chevron** button in the card's top bar (left of the type icon) to collapse the card to just its header bar. Click again to expand. The expanded height is remembered. Collapse is animated when animations are enabled.

### Minimap

A small overview of the entire canvas appears in the **top-right corner**. Click anywhere on the minimap to jump to that position; drag to pan continuously. Toggle visibility with `M` or in **Settings → General**.

### Canvas Navigation & Zoom

- **Pan:** Click + drag empty space, or scroll
- **Zoom:** `Ctrl + Scroll` to zoom toward the cursor; `Ctrl + +` / `Ctrl + -` to step in or out
- **Recenter + reset zoom:** `Ctrl + 0` — animates back to origin and restores 100% zoom
- **Zoom bar:** The pill widget in the bottom-right shows the current zoom level and provides one-click zoom in, reset, and zoom out

---

### Theme & Appearance

- **Sun / Moon** icon in the sidebar — toggle Dark / Light (animated diagonal wipe)
- **Settings → Appearance:**
  - Theme (Light / Dark)
  - Accent colour (10 choices)
  - Font family (multiple monospace fonts)
  - Font size

### Settings Overview

| Category | Contents |
|----------|----------|
| **General** | Sidebar open on start, animations, new-board button position, card delete confirmation, minimap |
| **Appearance** | Theme, accent colour, font family, font size |
| **Shortcuts** | Full keyboard shortcut reference (read-only) |
| **About** | App version, config path, debug mode toggle |

### Delete Confirmation

With **Confirm card deletion** enabled (default: on), deleting a card via the toolbar shows a confirmation dialog. Disable it in **Settings → General** for instant deletion.

---

## 🎨 Syntax Highlighting Languages

40+ languages including C, C++, Rust, Go, Python, JavaScript, TypeScript, HTML, CSS, JSON, YAML, SQL, Java, Kotlin, Swift, Bash, and more.

---

## 🏗️ Built With

- [Rust](https://www.rust-lang.org/) — Systems programming language
- [Iced](https://iced.rs/) — Cross-platform GUI library
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) — Markdown parser
- [syntect](https://github.com/trishume/syntect) — Syntax highlighting
- [Bootstrap Icons](https://icons.getbootstrap.com/) — Icon library
- [arboard](https://github.com/1Password/arboard) — System clipboard
- [dirs](https://github.com/dirs-dev/dirs-rs) — Platform config directories

---

## 🛠️ Configuration

| Platform | Path |
|----------|------|
| Linux    | `~/.config/cards/config.toml` |
| macOS    | `~/Library/Application Support/cards/config.toml` |
| Windows  | `%APPDATA%\cards\config.toml` |

### Self-Healing Config

Missing keys are added with defaults; obsolete keys are removed — automatically on every launch.

### Available Settings

| Key | Default | Description |
|-----|---------|-------------|
| `general.sidebar_open_on_start` | `true` | Open sidebar on launch |
| `general.enable_animations` | `true` | Enable all animations |
| `general.new_board_button_at_top` | `false` | Place new-board button at top of list |
| `general.confirm_card_delete` | `true` | Confirmation dialog before deleting a card |
| `general.show_minimap` | `true` | Show minimap overlay (toggle with `M`) |
| `general.debug_mode` | `false` | Print debug output to the console |
| `appearance.theme` | `light` | `light` or `dark` |
| `appearance.accent_color` | `purple` | Accent colour (`purple`, `blue`, `red`, `green`, `orange`, `pink`, `cyan`, `yellow`, `gray`, `coral`) |
| `appearance.font.family` | `jetbrainsmono` | Monospace font for card text |
| `appearance.font.size` | `14.0` | Font size in points |

---

## 📝 Tips & Tricks

1. **Quick card:** `N` drops a card at your cursor; canvas centre is used as fallback when the mouse is over the sidebar
2. **Auto-complete Markdown:** Type `<md>` + `>` and the closing tag inserts itself
3. **Syntax highlighting:** ` ```python ` opens a highlighted Python block
4. **Colour categories:** Assign colours + icons to group related cards visually
5. **Board-per-project:** Use separate boards for different projects; `Ctrl + Tab` to switch
6. **Inline rename:** Double-click any board to rename it
7. **Clickable checkboxes:** `- [ ]` / `- [x]` items are directly toggleable in rendered view
8. **Toolbar wrapping:** Select text → click a toolbar button to wrap it in markdown syntax
9. **Grid snapping:** Drag and resize both snap to the dot grid for perfect alignment
10. **Lost on canvas?** `Ctrl + 0` animates back to origin and resets zoom in one step
11. **Font tuning:** Settings → Appearance → Fonts for family and size
12. **Board delete button:** Hover a board in the sidebar to reveal the red delete button
13. **Multi-select:** Drag across empty canvas space to box-select, then move or delete all at once
14. **Duplicate shortcut:** `Ctrl + D` duplicates the selected card(s), preserving size and colour
15. **Collapse clutter:** Click the ▶ chevron on busy cards to fold them to header-only view
16. **Minimap navigation:** Click or drag the minimap to pan without scrolling across the canvas
17. **Move across boards:** Drag a card's header toward the sidebar to reveal the board drop zone

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🙏 Acknowledgments

- Built with the amazing [Iced](https://iced.rs/) framework
- Icons provided by [Bootstrap Icons](https://icons.getbootstrap.com/)

## 📧 Contact

For questions or feedback, please open an issue on GitHub.

---

**Made with ❤️ and Rust**
