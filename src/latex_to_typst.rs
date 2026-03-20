/// Convert LaTeX math notation to Typst math notation.
///
/// Handles: Greek letters, math operators, fractions, roots, accents,
/// font commands, environments (matrix, cases, align), \left/\right pairs,
/// subscripts/superscripts with multi-token braces, and more.
pub fn convert(latex: &str) -> String {
    let input = normalise(latex);
    let mut ctx = Ctx::new(&input);
    ctx.parse_expr()
}

// ── Normalisation ─────────────────────────────────────────────────────────────

/// Collapse runs of whitespace and strip surrounding whitespace.
fn normalise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch == '\n' || ch == '\t' { // treat as space
            if !prev_space { out.push(' '); prev_space = true; }
        } else if ch == ' ' {
            if !prev_space { out.push(' '); prev_space = true; }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

// ── Parser context ────────────────────────────────────────────────────────────

struct Ctx<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Ctx<'a> {
    fn new(src: &'a str) -> Self { Self { src, pos: 0 } }

    fn peek(&self) -> Option<char> { self.src[self.pos..].chars().next() }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() { self.pos += ch.len_utf8(); }
    }

    fn eat_whitespace(&mut self) {
        while self.peek() == Some(' ') { self.advance(); }
    }

    fn remaining(&self) -> &str { &self.src[self.pos..] }

    /// Read an ASCII alphabetic identifier (command name after backslash).
    fn read_ident(&mut self) -> &str {
        let start = self.pos;
        while self.peek().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
            self.advance();
        }
        &self.src[start..self.pos]
    }

    /// Consume a balanced `{...}` and return its inner content (without braces).
    /// Returns empty string if the next char is not `{`.
    fn extract_braced(&mut self) -> String {
        self.eat_whitespace();
        if self.peek() != Some('{') { return String::new(); }
        self.advance(); // skip `{`
        let mut depth = 1usize;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('{') => { depth += 1; out.push('{'); self.advance(); }
                Some('}') => {
                    depth -= 1;
                    self.advance();
                    if depth == 0 { break; }
                    out.push('}');
                }
                Some(c) => { out.push(c); self.advance(); }
            }
        }
        out
    }

    /// Consume an optional `[...]` and return its inner content, or None.
    fn extract_optional(&mut self) -> Option<String> {
        self.eat_whitespace();
        if self.peek() != Some('[') { return None; }
        self.advance(); // skip `[`
        let mut depth = 1usize;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('[') => { depth += 1; out.push('['); self.advance(); }
                Some(']') => {
                    depth -= 1;
                    self.advance();
                    if depth == 0 { break; }
                    out.push(']');
                }
                Some(c) => { out.push(c); self.advance(); }
            }
        }
        Some(out)
    }

    /// Top-level expression parser — returns full Typst math string.
    fn parse_expr(&mut self) -> String {
        let mut out = String::new();
        loop {
            self.eat_whitespace();
            match self.peek() {
                None | Some('}') => break,
                Some(_) => {
                    let token = self.parse_token();
                    if !token.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ')
                            && !token.starts_with(' ')
                            && needs_space_before(&token)
                            && needs_space_after(&out)
                        {
                            out.push(' ');
                        }
                        out.push_str(&token);
                    }
                }
            }
        }
        out
    }

    fn parse_token(&mut self) -> String {
        self.eat_whitespace();
        match self.peek() {
            None => String::new(),
            Some('\\') => {
                self.advance(); // consume backslash
                self.parse_command()
            }
            Some('^') => { self.advance(); self.parse_sup_sub('^') }
            Some('_') => { self.advance(); self.parse_sup_sub('_') }
            Some('{') => {
                // Bare brace group — just convert the contents
                let inner = self.extract_braced();
                let converted = convert(&inner);
                // If the converted result has spaces (multi-token), wrap it
                if converted.contains(' ') || converted.contains('/') {
                    format!("({})", converted)
                } else {
                    converted
                }
            }
            Some('&') => { self.advance(); " & ".to_string() }
            Some(ch) => {
                self.advance();
                ch.to_string()
            }
        }
    }

    /// Handle `^` or `_` followed by either a single char or `{...}`.
    fn parse_sup_sub(&mut self, op: char) -> String {
        self.eat_whitespace();
        let arg = if self.peek() == Some('{') {
            let inner = self.extract_braced();
            let converted = convert(&inner);
            // Only add parens when the result is multi-token
            if is_single_typst_atom(&converted) {
                converted
            } else {
                format!("({})", converted)
            }
        } else {
            // Single-char argument (possibly a command: e.g. ^\infty)
            self.parse_token()
        };
        format!("{}{}", op, arg)
    }

    /// Parse the command name that follows a `\`.
    fn parse_command(&mut self) -> String {
        // Single non-alpha chars like `\,` `\;` `\\` `\{` `\}`
        if self.peek().map(|c| !c.is_ascii_alphabetic()).unwrap_or(true) {
            let ch = match self.peek() {
                Some(',') | Some(';') | Some('!') => { self.advance(); return " ".to_string(); }
                Some(' ') => { self.advance(); return " ".to_string(); }
                Some('\\') => { self.advance(); return " \\ ".to_string(); } // line break → space in Typst
                Some('{') => { self.advance(); return "{".to_string(); }
                Some('}') => { self.advance(); return "}".to_string(); }
                Some('[') => { self.advance(); return "[".to_string(); }
                Some(']') => { self.advance(); return "]".to_string(); }
                Some('|') => { self.advance(); return "|".to_string(); }
                Some('/') => { self.advance(); return "/".to_string(); }
                Some('.') => { self.advance(); return ".".to_string(); }
                Some('-') => { self.advance(); return "-".to_string(); }
                Some(c) => { let s = c.to_string(); self.advance(); return s; }
                None => return String::new(),
            };
            return ch;
        }

        let name = self.read_ident().to_string();
        self.eat_whitespace();

        match name.as_str() {
            // ── Fractions ───────────────────────────────────────────────────
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let num = self.extract_braced();
                let den = self.extract_braced();
                format!("({})/({})", convert(&num), convert(&den))
            }
            "nicefrac" | "sfrac" => {
                let num = self.extract_braced();
                let den = self.extract_braced();
                format!("{}/{}", convert(&num), convert(&den))
            }

            // ── Roots ───────────────────────────────────────────────────────
            "sqrt" => {
                let opt = self.extract_optional();
                let arg = self.extract_braced();
                if let Some(n) = opt {
                    format!("root({}, {})", convert(&n), convert(&arg))
                } else {
                    format!("sqrt({})", convert(&arg))
                }
            }

            // ── Binomial ────────────────────────────────────────────────────
            "binom" | "dbinom" | "tbinom" => {
                let n = self.extract_braced();
                let k = self.extract_braced();
                format!("binom({}, {})", convert(&n), convert(&k))
            }
            "choose" => "binom".to_string(), // \choose is weird TeX

            // ── Text / font commands ─────────────────────────────────────────
            "text" | "mbox" | "textrm" | "textit" | "textbf" => {
                let arg = self.extract_braced();
                // Wrap in quotes for Typst math text
                format!("\"{}\"", arg)
            }
            "operatorname" | "DeclareMathOperator" => {
                let arg = self.extract_braced();
                format!("op(\"{}\")", arg)
            }
            "mathbf" | "boldsymbol" | "bm" => {
                let arg = self.extract_braced();
                format!("bold({})", convert(&arg))
            }
            "mathit" => {
                let arg = self.extract_braced();
                format!("italic({})", convert(&arg))
            }
            "mathrm" | "mathup" => {
                let arg = self.extract_braced();
                format!("upright({})", convert(&arg))
            }
            "mathsf" | "textsf" => {
                let arg = self.extract_braced();
                format!("sans({})", convert(&arg))
            }
            "mathtt" | "texttt" => {
                let arg = self.extract_braced();
                format!("mono({})", convert(&arg))
            }
            "mathcal" | "mathscr" => {
                let arg = self.extract_braced();
                format!("cal({})", convert(&arg))
            }
            "mathfrak" => {
                let arg = self.extract_braced();
                format!("frak({})", convert(&arg))
            }
            "mathbb" => {
                let arg = self.extract_braced();
                // Map single letters to Typst's double-struck names
                match arg.trim() {
                    "N" => "NN".to_string(),
                    "Z" => "ZZ".to_string(),
                    "Q" => "QQ".to_string(),
                    "R" => "RR".to_string(),
                    "C" => "CC".to_string(),
                    "H" => "HH".to_string(),
                    other => format!("bb({})", convert(other)),
                }
            }

            // ── Accents ─────────────────────────────────────────────────────
            "hat"   => { let a = self.extract_braced(); format!("hat({})", convert(&a)) }
            "check" | "v" => { let a = self.extract_braced(); format!("caron({})", convert(&a)) }
            "breve" => { let a = self.extract_braced(); format!("breve({})", convert(&a)) }
            "acute" => { let a = self.extract_braced(); format!("acute({})", convert(&a)) }
            "grave" => { let a = self.extract_braced(); format!("grave({})", convert(&a)) }
            "tilde" => { let a = self.extract_braced(); format!("tilde({})", convert(&a)) }
            "bar" | "overline" => { let a = self.extract_braced(); format!("overline({})", convert(&a)) }
            "underline" => { let a = self.extract_braced(); format!("underline({})", convert(&a)) }
            "vec"   => { let a = self.extract_braced(); format!("arrow({})", convert(&a)) }
            "dot"   => { let a = self.extract_braced(); format!("dot({})", convert(&a)) }
            "ddot"  => { let a = self.extract_braced(); format!("dot.double({})", convert(&a)) }
            "dddot" => { let a = self.extract_braced(); format!("dot.triple({})", convert(&a)) }
            "ddddot" => { let a = self.extract_braced(); format!("dot.quad({})", convert(&a)) }
            "widetilde" => { let a = self.extract_braced(); format!("tilde({})", convert(&a)) }
            "widehat"   => { let a = self.extract_braced(); format!("hat({})", convert(&a)) }
            "overbrace"  => { let a = self.extract_braced(); format!("overbrace({})", convert(&a)) }
            "underbrace" => { let a = self.extract_braced(); format!("underbrace({})", convert(&a)) }
            "overset" => {
                let top = self.extract_braced();
                let base = self.extract_braced();
                format!("overset({}, {})", convert(&top), convert(&base))
            }
            "underset" => {
                let bot = self.extract_braced();
                let base = self.extract_braced();
                format!("underset({}, {})", convert(&bot), convert(&base))
            }
            "stackrel" => {
                let top = self.extract_braced();
                let base = self.extract_braced();
                format!("overset({}, {})", convert(&top), convert(&base))
            }
            "xrightarrow" => {
                self.extract_optional(); // skip optional
                let arg = self.extract_braced();
                format!("xarrow(sym: ->, {})", convert(&arg))
            }
            "xleftarrow" => {
                self.extract_optional();
                let arg = self.extract_braced();
                format!("xarrow(sym: <-, {})", convert(&arg))
            }

            // ── Limits / integral decorations ────────────────────────────────
            "limits"   => "limits".to_string(),
            "nolimits" => "".to_string(),
            "displaylimits" => "".to_string(),

            // ── Big operators (with limits) ───────────────────────────────────
            // These map directly to Typst names
            "sum"        => "sum".to_string(),
            "prod"       => "product".to_string(),
            "coprod"     => "product.co".to_string(),
            "int"        => "integral".to_string(),
            "iint"       => "integral.double".to_string(),
            "iiint"      => "integral.triple".to_string(),
            "oint"       => "integral.cont".to_string(),
            "oiint"      => "integral.surf".to_string(),
            "oiiint"     => "integral.vol".to_string(),
            "bigcap"     => "sect.big".to_string(),
            "bigcup"     => "union.big".to_string(),
            "bigsqcup"   => "union.sq.big".to_string(),
            "bigvee"     => "or.big".to_string(),
            "bigwedge"   => "and.big".to_string(),
            "bigotimes"  => "times.circle.big".to_string(),
            "bigoplus"   => "plus.circle.big".to_string(),
            "biguplus"   => "union.plus.big".to_string(),

            // ── \left / \right ───────────────────────────────────────────────
            "left" => {
                let delim = self.next_delimiter();
                // collect until matching \right
                let (inner, close_delim) = self.collect_until_right();
                let typst_open  = latex_delim_to_typst(&delim);
                let typst_close = latex_delim_to_typst(&close_delim);
                if typst_open == "(" && typst_close == ")" {
                    format!("lr(({}))", convert(&inner))
                } else if typst_open == "[" && typst_close == "]" {
                    format!("lr([{}])", convert(&inner))
                } else if typst_open == "|" && typst_close == "|" {
                    format!("lr(|{}|)", convert(&inner))
                } else if typst_open == "||" && typst_close == "||" {
                    format!("lr(||{}||)", convert(&inner))
                } else if typst_open == "{" && typst_close == "}" {
                    format!("lr({{{}}}, {})", convert(&inner), "")
                        .replace(", )", ")")  // trim trailing comma
                } else if typst_open == "." || typst_close == "." {
                    // \left. or \right. means no delimiter
                    convert(&inner)
                } else {
                    format!("lr({}{}{}, {})", typst_open, convert(&inner), typst_close, "")
                        .replace(", )", ")")
                }
            }
            // Absorb stray \right — shouldn't normally appear alone
            "right" => {
                self.next_delimiter();
                String::new()
            }

            // ── Environments ─────────────────────────────────────────────────
            "begin" => {
                let env = self.extract_braced();
                self.parse_environment(&env)
            }
            "end" => {
                self.extract_braced(); // consume env name
                String::new()
            }

            // ── Spacing commands (collapse to single space) ──────────────────
            "quad" => "  ".to_string(),
            "qquad" => "   ".to_string(),
            "," | ";" | ":" | "!" | " " => " ".to_string(),
            "mkern" | "kern" | "hspace" | "hskip" => {
                self.extract_braced();
                " ".to_string()
            }
            "phantom" => { self.extract_braced(); " ".to_string() }

            // ── Differentials ────────────────────────────────────────────────
            // \mathrm{d} and \mathsf{d} are handled above; bare \d is not standard
            "d" => "dif ".to_string(),

            // ── Display/inline math switchers (ignored here) ─────────────────
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle"
            | "normalsize" | "small" | "large" => String::new(),

            // ── Cases/piecewise ──────────────────────────────────────────────
            // handled via \begin{cases}

            // ── Delimiters written as commands ───────────────────────────────
            "langle"  => "angle.l ".to_string(),
            "rangle"  => "angle.r ".to_string(),
            "lfloor"  => "floor.l ".to_string(),
            "rfloor"  => "floor.r ".to_string(),
            "lceil"   => "ceil.l ".to_string(),
            "rceil"   => "ceil.r ".to_string(),
            "lbrace" | "lBrace" => "{ ".to_string(),
            "rbrace" | "rBrace" => "} ".to_string(),
            "vert"    => "| ".to_string(),
            "Vert"    => "|| ".to_string(),
            "lvert"   => "| ".to_string(),
            "rvert"   => "| ".to_string(),
            "lVert"   => "|| ".to_string(),
            "rVert"   => "|| ".to_string(),

            // ── Misc ─────────────────────────────────────────────────────────
            "not" => {
                // \not\in → in.not etc.  Just apply to next token
                let next = self.parse_token();
                negate_symbol(&next)
            }
            "tag" | "label" | "nonumber" | "notag" => {
                self.extract_braced();
                String::new()
            }
            "ref" | "eqref" => {
                self.extract_braced();
                String::new()
            }
            "hline" | "cline" => String::new(),
            "notin" => "in.not ".to_string(),

            // ── Default: look up in symbol table ─────────────────────────────
            other => {
                if let Some(typst) = symbol(other) {
                    typst.to_string()
                } else {
                    // Unknown command — pass through without backslash as-is
                    // (Typst might understand it, e.g. sin, cos are built-in)
                    other.to_string()
                }
            }
        }
    }

    /// Read a single LaTeX delimiter token after `\left` or `\right`.
    fn next_delimiter(&mut self) -> String {
        self.eat_whitespace();
        match self.peek() {
            Some('\\') => {
                self.advance();
                let id = self.read_ident().to_string();
                if id.is_empty() {
                    // e.g. \{ or \}
                    if let Some(c) = self.peek() {
                        self.advance();
                        return format!("\\{}", c);
                    }
                }
                format!("\\{}", id)
            }
            Some(c) => { self.advance(); c.to_string() }
            None => ".".to_string(),
        }
    }

    /// Collect content up to (and consuming) the matching `\right`.
    /// Returns (inner_content, closing_delimiter_string).
    fn collect_until_right(&mut self) -> (String, String) {
        let mut depth = 1usize; // nesting for \left / \right pairs
        let mut inner = String::new();
        let mut close = ".".to_string();

        loop {
            match self.peek() {
                None => break,
                Some('\\') => {
                    self.advance();
                    let id = self.read_ident().to_string();
                    if id == "right" {
                        depth -= 1;
                        if depth == 0 {
                            close = self.next_delimiter();
                            break;
                        }
                        inner.push_str("\\right");
                        inner.push_str(&id[5..]); // shouldn't happen
                    } else if id == "left" {
                        depth += 1;
                        inner.push_str("\\left");
                        let delim = self.next_delimiter();
                        inner.push_str(&delim);
                    } else {
                        inner.push('\\');
                        inner.push_str(&id);
                    }
                }
                Some(c) => { inner.push(c); self.advance(); }
            }
        }
        (inner, close)
    }

    /// Parse a `\begin{env}...\end{env}` block. `env` is already extracted.
    fn parse_environment(&mut self, env: &str) -> String {
        // Collect raw content until \end{env}
        let raw = self.collect_until_end(env);

        match env {
            "matrix" | "matrix*" => format_matrix(&raw, "mat(delim: #none,", ")"),
            "pmatrix" | "pmatrix*" => format_matrix(&raw, "mat(", ")"),
            "bmatrix" | "bmatrix*" => format_matrix(&raw, "mat(delim: \"[\",", ")"),
            "Bmatrix" | "Bmatrix*" => format_matrix(&raw, "mat(delim: \"{\",", ")"),
            "vmatrix" | "vmatrix*" => format_matrix(&raw, "mat(delim: \"|\",", ")"),
            "Vmatrix" | "Vmatrix*" => format_matrix(&raw, "mat(delim: \"||\",", ")"),
            "smallmatrix" => format_matrix(&raw, "mat(delim: #none,", ")"),
            "cases" | "cases*" => format_cases(&raw),
            "dcases" => format_cases(&raw),
            "rcases" => format_cases(&raw),
            "align" | "align*" | "aligned" | "alignat" | "alignat*"
            | "eqnarray" | "eqnarray*" => format_aligned(&raw),
            "gather" | "gather*" | "gathered" => format_aligned(&raw),
            "multline" | "multline*" => convert(&raw.replace("\\\\", " ")),
            "split" => convert(&raw.replace("\\\\", " ")),
            "equation" | "equation*" | "math" | "displaymath" => convert(&raw),
            "subarray" => { self.extract_optional(); convert(&raw) }
            "array" => {
                self.extract_braced(); // column spec
                format_matrix(&raw, "mat(delim: #none,", ")")
            }
            _ => {
                // Unknown environment — just convert the contents
                convert(&raw)
            }
        }
    }

    /// Collect raw text until `\end{env_name}`.
    fn collect_until_end(&mut self, env: &str) -> String {
        let mut out = String::new();
        loop {
            // Look for \end
            if self.remaining().starts_with("\\end") {
                // Peek at what follows \end
                let saved = self.pos;
                self.pos += 4; // skip \end
                self.eat_whitespace();
                let next_env = self.extract_braced();
                if next_env == env {
                    break; // found our closing \end
                } else {
                    // Not our end — put it all back into output
                    let consumed = &self.src[saved..self.pos];
                    out.push_str(consumed);
                }
            } else {
                match self.peek() {
                    None => break,
                    Some(c) => { out.push(c); self.advance(); }
                }
            }
        }
        out
    }
}

// ── Matrix / cases / align formatters ────────────────────────────────────────

fn format_matrix(raw: &str, open: &str, close: &str) -> String {
    // Rows separated by `\\`, columns by `&`
    let rows: Vec<&str> = raw.split("\\\\").collect();
    let mut typst_rows: Vec<String> = Vec::new();
    for row in &rows {
        let trimmed = row.trim();
        if trimmed.is_empty() { continue; }
        let cols: Vec<String> = trimmed.split('&')
            .map(|c| convert(c.trim()))
            .collect();
        typst_rows.push(cols.join(", "));
    }
    format!("{} {} {}", open, typst_rows.join("; "), close)
}

fn format_cases(raw: &str) -> String {
    let rows: Vec<&str> = raw.split("\\\\").collect();
    let mut cases: Vec<String> = Vec::new();
    for row in &rows {
        let trimmed = row.trim();
        if trimmed.is_empty() { continue; }
        // Split on & for condition/value
        let parts: Vec<String> = trimmed.split('&').map(|p| convert(p.trim())).collect();
        cases.push(parts.join(" quad "));
    }
    format!("cases({})", cases.join(", "))
}

fn format_aligned(raw: &str) -> String {
    // Join all rows with line-break operator in Typst
    let rows: Vec<&str> = raw.split("\\\\").collect();
    let typst_rows: Vec<String> = rows.iter()
        .map(|r| {
            let r = r.trim();
            // Replace `&=` / `& =` alignment markers
            convert(&r.replace("& =", "=").replace("&=", "=").replace("&", " "))
        })
        .filter(|r| !r.is_empty())
        .collect();
    typst_rows.join(" \\ ")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a LaTeX delimiter to its Typst equivalent.
fn latex_delim_to_typst(d: &str) -> &str {
    match d {
        "(" => "(",
        ")" => ")",
        "[" | "\\[" => "[",
        "]" | "\\]" => "]",
        "\\{" | "lbrace" | "\\lbrace" => "{",
        "\\}" | "rbrace" | "\\rbrace" => "}",
        "|" | "\\vert" | "\\lvert" | "\\rvert" => "|",
        "\\|" | "\\Vert" | "\\lVert" | "\\rVert" => "||",
        "\\langle" => "⟨",
        "\\rangle" => "⟩",
        "\\lfloor" => "⌊",
        "\\rfloor" => "⌋",
        "\\lceil"  => "⌈",
        "\\rceil"  => "⌉",
        "." => ".",
        _ => d,
    }
}

/// Attempt to negate a Typst symbol (for `\not`).
fn negate_symbol(s: &str) -> String {
    match s.trim() {
        "in" | "in " => "in.not ".to_string(),
        "=" | "= " => "eq.not ".to_string(),
        "<" | "< " => "lt.not ".to_string(),
        ">" | "> " => "gt.not ".to_string(),
        "lt.eq" | "lt.eq " => "lt.eq.not ".to_string(),
        "gt.eq" | "gt.eq " => "gt.eq.not ".to_string(),
        "subset" | "subset " => "subset.not ".to_string(),
        "supset" | "supset " => "supset.not ".to_string(),
        "tilde.op" | "tilde.op " => "tilde.not ".to_string(),
        _ => format!("cancel({})", s),
    }
}

/// Returns `true` when the Typst atom is a single logical unit
/// (so subscripts/superscripts don't need extra parens).
fn is_single_typst_atom(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() { return true; }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 1 { return true; }
    // Single word (no spaces, no operators except dots in names)
    if !s.contains(' ') && !s.contains('+') && !s.contains('-')
        && !s.contains('*') && !s.contains('/') && !s.contains(',')
    {
        return true;
    }
    false
}

fn needs_space_before(token: &str) -> bool {
    !matches!(token.chars().next(), Some('^') | Some('_') | Some(',') | Some('.') | Some(')') | Some(']') | Some('}'))
}

fn needs_space_after(out: &str) -> bool {
    !matches!(out.chars().last(), Some('(') | Some('[') | Some('{') | Some(' '))
}

// ── Symbol table ──────────────────────────────────────────────────────────────

fn symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        // Greek lowercase
        "alpha"      => "alpha",   "beta"       => "beta",    "gamma"  => "gamma",
        "delta"      => "delta",   "epsilon"    => "epsilon", "varepsilon" => "epsilon.alt",
        "zeta"       => "zeta",    "eta"        => "eta",     "theta"  => "theta",
        "vartheta"   => "theta.alt","iota"      => "iota",    "kappa"  => "kappa",
        "lambda"     => "lambda",  "mu"         => "mu",      "nu"     => "nu",
        "xi"         => "xi",      "pi"         => "pi",      "varpi"  => "pi.alt",
        "rho"        => "rho",     "varrho"     => "rho.alt", "sigma"  => "sigma",
        "varsigma"   => "sigma.alt","tau"       => "tau",     "upsilon" => "upsilon",
        "phi"        => "phi.alt", "varphi"     => "phi",     "chi"    => "chi",
        "psi"        => "psi",     "omega"      => "omega",

        // Greek uppercase
        "Gamma"  => "Gamma",  "Delta"  => "Delta",   "Theta"  => "Theta",
        "Lambda" => "Lambda", "Xi"     => "Xi",      "Pi"     => "Pi",
        "Sigma"  => "Sigma",  "Upsilon" => "Upsilon","Phi"    => "Phi",
        "Psi"    => "Psi",    "Omega"  => "Omega",

        // Binary operators
        "pm"         => "plus.minus",   "mp"         => "minus.plus",
        "times"      => "times",        "div"        => "div",
        "cdot"       => "dot.op",       "bullet"     => "bullet",
        "ast"        => "ast",          "star"       => "star",
        "circ"       => "circle.stroked.small",
        "oplus"      => "plus.circle",  "ominus"     => "minus.circle",
        "otimes"     => "times.circle", "oslash"     => "slash.circle",
        "odot"       => "dot.circle",   "wedge" | "land" => "and",
        "vee"   | "lor"  => "or",       "cap"        => "sect",
        "cup"        => "union",        "sqcap"      => "sect.sq",
        "sqcup"      => "union.sq",     "uplus"      => "union.plus",
        "amalg"      => "product.co",   "setminus"   => "without",
        "wr"         => "wreath",

        // Relations
        "leq" | "le"  => "lt.eq",       "geq" | "ge"  => "gt.eq",
        "neq" | "ne"  => "eq.not",      "approx"     => "approx",
        "equiv"       => "equiv",        "sim"        => "tilde.op",
        "simeq"       => "tilde.eq",     "cong"       => "tilde.equiv",
        "propto"      => "prop",         "asymp"      => "asymp",
        "ll"          => "lt.double",    "gg"         => "gt.double",
        "subset"      => "subset",       "supset"     => "supset",
        "subseteq"    => "subset.eq",    "supseteq"   => "supset.eq",
        "subsetneq"   => "subset.neq",   "supsetneq"  => "supset.neq",
        "sqsubset"    => "subset.sq",    "sqsupset"   => "supset.sq",
        "sqsubseteq"  => "subset.sq.eq", "sqsupseteq" => "supset.sq.eq",
        "in"          => "in",           "notin"      => "in.not",
        "ni"          => "in.rev",       "nmid"       => "divides.not",
        "mid"         => "divides",      "parallel"   => "parallel",
        "perp"        => "perp",         "bowtie"     => "bowtie",
        "models"      => "models",       "prec"       => "prec",
        "succ"        => "succ",         "preceq"     => "prec.eq",
        "succeq"      => "succ.eq",

        // Arrows
        "to" | "rightarrow"    => "->",   "leftarrow"     => "<-",
        "Rightarrow"           => "=>",   "Leftarrow"     => "<=",
        "leftrightarrow"       => "<->",  "Leftrightarrow" | "iff" => "<=>",
        "longrightarrow"       => "-->",  "longleftarrow"  => "<--",
        "Longrightarrow"       => "==>",  "Longleftarrow"  => "<==",
        "longleftrightarrow"   => "<-->", "Longleftrightarrow" => "<==>",
        "implies"              => "=>",   "impliedby"      => "<=",
        "nearrow"  => "arrow.tr",   "nwarrow" => "arrow.tl",
        "searrow"  => "arrow.br",   "swarrow" => "arrow.bl",
        "uparrow"  => "arrow.t",    "downarrow" => "arrow.b",
        "Uparrow"  => "arrow.t.double", "Downarrow" => "arrow.b.double",
        "updownarrow" => "arrow.tb", "Updownarrow" => "arrow.tb.double",
        "mapsto"   => "arrow.r.bar",
        "hookleftarrow"  => "arrow.l.hook",
        "hookrightarrow" => "arrow.r.hook",
        "leftharpoonup"   => "harpoon.lt",
        "leftharpoondown" => "harpoon.lb",
        "rightharpoonup"  => "harpoon.rt",
        "rightharpoondown"=> "harpoon.rb",

        // Logic
        "forall"     => "forall",   "exists"  => "exists",
        "nexists"    => "exists.not","neg" | "lnot" => "not",
        "land"       => "and",       "lor"    => "or",
        "top"        => "top",       "bot"    => "bot",
        "therefore"  => "therefore", "because" => "because",

        // Misc symbols
        "infty"      => "infinity",   "partial"  => "diff",
        "nabla"      => "nabla",      "hbar"     => "planck.reduce",
        "ell"        => "ell",        "wp"       => "weierp",
        "Re"         => "Re",         "Im"       => "Im",
        "aleph"      => "aleph",      "beth"     => "beth",
        "gimel"      => "gimel",      "daleth"   => "daleth",
        "emptyset" | "varnothing" => "emptyset",
        "angle"      => "angle",      "measuredangle" => "angle.arc",
        "triangle"   => "triangle.stroked.t",
        "square"     => "square.stroked",
        "diamond"    => "diamond.stroked",
        "lozenge"    => "lozenge.stroked",
        "clubsuit"   => "suit.club",  "diamondsuit" => "suit.diamond",
        "heartsuit"  => "suit.heart", "spadesuit"   => "suit.spade",
        "sharp"      => "sharp",      "flat"     => "flat",
        "natural"    => "natural",

        // Dots
        "ldots" | "dots" | "dotsc"  => "...",
        "cdots" | "dotsb" | "dotsi" => "dots.h",
        "vdots"                      => "dots.v",
        "ddots"                      => "dots.down",
        "udots"                      => "dots.up",
        "iddots"                     => "dots.up",

        // Functions (just strip backslash — Typst has them built in)
        "sin"    => "sin",    "cos"    => "cos",    "tan"    => "tan",
        "csc"    => "csc",    "sec"    => "sec",    "cot"    => "cot",
        "arcsin" => "arcsin", "arccos" => "arccos", "arctan" => "arctan",
        "sinh"   => "sinh",   "cosh"   => "cosh",   "tanh"   => "tanh",
        "coth"   => "coth",
        "log"    => "log",    "ln"     => "ln",     "exp"    => "exp",
        "max"    => "max",    "min"    => "min",
        "sup"    => "sup",    "inf"    => "inf",    "lim"    => "lim",
        "limsup" => "limsup", "liminf" => "liminf",
        "det"    => "det",    "ker"    => "ker",    "dim"    => "dim",
        "deg"    => "deg",    "gcd"    => "gcd",    "lcm"    => "lcm",
        "arg"    => "arg",    "hom"    => "hom",    "Hom"    => "Hom",
        "tr"     => "tr",     "rank"   => "rank",   "diag"   => "diag",
        "sgn"    => "sgn",    "mod"    => "mod",    "grad"   => "grad",
        "curl"   => "curl",

        // Misc
        "prime"      => "prime",
        "dagger"     => "dagger",    "ddagger" => "dagger.double",
        "wr"         => "wreath",    "S"       => "section",
        "P"          => "pilcrow",   "pounds"  => "pound",
        "copyright"  => "copyright", "dag"     => "dagger",
        "ddag"       => "dagger.double",
        "checkmark"  => "checkmark", "surd"   => "sqrt(\"\")",
        "colon"      => ":",

        _ => return None,
    })
}
