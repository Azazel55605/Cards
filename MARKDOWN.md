# Markdown in Cards

Cards supports a rich subset of Markdown inside any **Markdown card**. This document covers every supported element, with examples you can paste directly into a card.

---

## Card Types

There are two card types:

- **Text card** — plain text with `[[card reference]]` link support only.
- **Markdown card** — full Markdown rendering. Everything in this document applies to Markdown cards.

To switch a card to Markdown mode, right-click the card and choose *Change type → Markdown*, or use the card toolbar.

---

## Headings

```
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

Headings are rendered bold and progressively smaller from H1 (largest) down to H6. Use `#` followed by a space at the start of a line.

---

## Emphasis

```
**bold**
*italic*
***bold and italic***
~~strikethrough~~
```

These can be combined: `***~~all four~~***`.

---

## Lists

### Unordered Lists

```
- First item
- Second item
  - Nested item (2 spaces)
    - Deeper nesting
```

Bullet style changes by nesting depth: `•` → `◦` → `▸`.

### Ordered Lists

```
1. First item
2. Second item
3. Third item
```

Numbers are rendered automatically; the actual numbers you type do not need to be sequential.

### Nested Mixed Lists

```
1. Step one
   - Sub-item A
   - Sub-item B
2. Step two
   1. Ordered sub-item
   2. Another
```

---

## Task Lists

```
- [ ] Unchecked task
- [x] Completed task
- [ ] Another task
```

Checkboxes are interactive — click them to toggle checked/unchecked state. The state is saved to the card's content.

---

## Links

```
[Link text](https://example.com)
```

Links are rendered in blue with an underline. Clicking a link opens it in your default browser.

---

## Card References

```
[[Card Title]]
[[Board Name / Card Title]]
```

References link to other cards in your workspace. Clicking a reference jumps to that card, switching boards if necessary. The title match is case-insensitive.

---

## Code

### Inline Code

```
Use `backticks` for inline code.
```

Inline code is rendered in a monospace font with a subtle background highlight.

### Fenced Code Blocks

````
```rust
fn main() {
    println!("Hello, world!");
}
```
````

Supported language tokens include `rust`, `python`, `js`, `ts`, `go`, `java`, `c`, `cpp`, `css`, `html`, `json`, `toml`, `yaml`, `sh`, `bash`, and many more. Syntax highlighting uses the **base16-ocean.dark** theme.

For a plain (unhighlighted) block, omit the language or use no token:

````
```
plain text block
no highlighting
```
````

---

## Math

Cards renders math using the **Typst** typesetting engine, producing properly typeset vector output directly in the card. **Both LaTeX and Typst math syntax are accepted** — the app converts LaTeX to Typst automatically.

### Inline Math

```
The quadratic formula is $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.
```

Use single dollar signs `$...$` for inline math (within a paragraph).

### Display Math

```
$$
\int_0^{\infty} e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}
$$
```

Double dollar signs `$$...$$` create a centred display block.

### Math Code Block

````
```math
E = mc^2
```
````

A fenced block tagged `math` is treated identically to a `$$` display block.

### Supported LaTeX Commands

| Category         | Examples                                                             |
|------------------|----------------------------------------------------------------------|
| Fractions        | `\frac{a}{b}`, `\dfrac`, `\tfrac`                                   |
| Roots            | `\sqrt{x}`, `\sqrt[n]{x}`                                           |
| Greek letters    | `\alpha`, `\beta`, `\gamma`, `\delta`, `\omega`, `\Gamma`, `\Sigma` |
| Superscript      | `x^{2}`, `e^{i\pi}`                                                 |
| Subscript        | `x_{n}`, `a_{ij}`                                                   |
| Sums / products  | `\sum_{i=0}^{n}`, `\prod_{k=1}^{N}`                                 |
| Integrals        | `\int_0^\infty`, `\iint`, `\iiint`, `\oint`                         |
| Limits           | `\lim_{x \to 0}`, `\limsup`, `\liminf`                              |
| Trig / log       | `\sin`, `\cos`, `\tan`, `\ln`, `\log`, `\exp`                       |
| Arrows           | `\to`, `\rightarrow`, `\Rightarrow`, `\iff`, `\implies`             |
| Relations        | `\leq`, `\geq`, `\neq`, `\approx`, `\equiv`, `\in`, `\subset`      |
| Operators        | `\pm`, `\times`, `\div`, `\cdot`, `\oplus`, `\otimes`               |
| Logic            | `\forall`, `\exists`, `\neg`, `\land`, `\lor`                       |
| Misc             | `\infty`, `\partial`, `\nabla`, `\hbar`, `\emptyset`                |
| Accents          | `\hat{x}`, `\bar{x}`, `\vec{x}`, `\tilde{x}`, `\dot{x}`           |
| Delimiters       | `\left( \right)`, `\left[ \right]`, `\left\{ \right\}`             |
| Fonts            | `\mathbf{x}`, `\mathit{x}`, `\mathbb{R}`, `\mathcal{L}`            |
| Text             | `\text{hello}`                                                       |
| Matrices         | `\begin{pmatrix} a & b \\ c & d \end{pmatrix}`                      |
| Cases            | `\begin{cases} x & x > 0 \\ 0 & \text{otherwise} \end{cases}`      |
| Dots             | `\ldots`, `\cdots`, `\vdots`, `\ddots`                              |
| Binomial         | `\binom{n}{k}`                                                       |

---

## Blockquotes

```
> This is a blockquote.
>
> It can span multiple paragraphs.

> Outer quote
>> Nested one level
>>> Nested two levels
```

Each nesting level is indicated by a vertical bar on the left, one bar per level.

---

## Tables

```
| Column A | Column B | Column C |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

The header row is rendered **bold**. A horizontal separator rule is drawn beneath it. Columns are separated by `│` characters in the output.

---

## Horizontal Rules

```
---
```

Three or more hyphens on their own line render as a full-width horizontal rule with spacing above and below.

---

## Line Breaks

- A **soft break** (single newline in source) is collapsed to a space, following standard Markdown behaviour.
- A **hard break** (two trailing spaces, or `\` at end of line) forces a new line.
- A **blank line** between blocks creates paragraph spacing.

---

## Quick Reference

| Element         | Syntax                              |
|-----------------|-------------------------------------|
| Heading 1       | `# Title`                           |
| Heading 2       | `## Title`                          |
| Bold            | `**text**`                          |
| Italic          | `*text*`                            |
| Strikethrough   | `~~text~~`                          |
| Inline code     | `` `code` ``                        |
| Code block      | ` ```lang … ``` `                   |
| Inline math     | `$\LaTeX formula$`                  |
| Display math    | `$$\LaTeX formula$$`                |
| Unordered list  | `- item`                            |
| Ordered list    | `1. item`                           |
| Task            | `- [ ] task` / `- [x] done`        |
| Link            | `[text](url)`                       |
| Card reference  | `[[Card Title]]`                    |
| Blockquote      | `> text`                            |
| Table           | `\| A \| B \|` with separator row  |
| Horizontal rule | `---`                               |
