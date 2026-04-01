use yansi::{Color, Style};

use crate::{
    constants::Side,
    parse::syntax::{AtomKind, MatchKind, StringKind, TokenKind},
};
use std::collections::HashMap;

type StyleMap = HashMap<String, Style>;

/// Theme objects to allow customization of colors and styles
///
/// Themes represent a dark or light theme individually. We decide whether to
/// use light or dark at theme load time.
#[derive(Debug, Clone)]
pub(crate) struct Theme {
    /// the default background color for deleted lines
    pub(crate) novel_bg_left: Color,
    /// the default background color for added lines
    pub(crate) novel_bg_right: Color,
    pub(crate) base_style: Style,
    pub(crate) novel_style_left: Style,
    pub(crate) novel_style_right: Style,
    pub(crate) lineno_style_base: Style,
    pub(crate) lineno_style_left: Style,
    pub(crate) lineno_style_right: Style,
    pub(crate) styles: StyleMap,
}

impl Theme {
    pub(crate) fn default_style(&self, novel: bool, side: Side) -> &Style {
        match novel {
            true => match side {
                Side::Left => &self.novel_style_left,
                Side::Right => &self.novel_style_right,
            },
            false => &self.base_style,
        }
    }

    pub(crate) fn lineno_style(&self, novel: bool, side: Side) -> &Style {
        match novel {
            true => match side {
                Side::Left => &self.lineno_style_left,
                Side::Right => &self.lineno_style_right,
            },
            false => &self.lineno_style_base,
        }
    }

    /// This returns a style from the defined theme for a given kind, novelty, and side.
    ///
    /// Alternately, it attempts to fallback to an appropriate color so users can sparsely
    /// define themes.
    ///
    /// try to match <name>_<novel>_<side>
    /// try to match <name>_<novel>
    /// try to match <name>
    /// if none of these match, fallback to defaults
    pub(crate) fn style_by_type(&self, kind: &MatchKind, side: Side) -> &Style {
        // TODO: take syntax coloring preference into account as well as file type
        //  // Underline novel words inside comments in code, but
        //  // don't apply it to every single line in plaintext.
        //  if matches!(file_format, FileFormat::SupportedLanguage(_)) {
        //      style = style.underline();
        //  }

        // translate the status to strings
        let (status, token) = match kind {
            MatchKind::UnchangedToken { highlight, .. } => ("unchanged", highlight),
            MatchKind::Ignored { highlight } => ("ignored", highlight),
            MatchKind::Novel { highlight } => ("novel", highlight),
            MatchKind::UnchangedPartOfNovelItem { highlight, .. } => ("novel_line_part", highlight),
            MatchKind::NovelWord { highlight } => ("novel_word", highlight),
        };

        // translate the token kinds to strings
        let token_kind = match token {
            TokenKind::Delimiter => "delimiter",
            TokenKind::Atom(AtomKind::Normal) => "normal",
            TokenKind::Atom(AtomKind::String(StringKind::StringLiteral)) => "string_literal",
            TokenKind::Atom(AtomKind::String(StringKind::Text)) => "text",
            TokenKind::Atom(AtomKind::Type) => "type",
            TokenKind::Atom(AtomKind::Comment) => "comment",
            TokenKind::Atom(AtomKind::Keyword) => "keyword",
            TokenKind::Atom(AtomKind::Function) => "function",
            TokenKind::Atom(AtomKind::Variable) => "variable",
            TokenKind::Atom(AtomKind::Constant) => "constant",
            TokenKind::Atom(AtomKind::TreeSitterError) => "tree_sitter_error",
        };

        // translate the side to its corresponding name
        let side_name = match side {
            Side::Left => "left",
            Side::Right => "right",
        };

        // attempt to return the most specific style first
        if let Some(full_style) = self
            .styles
            .get(&format!("{}_{}_{}", token_kind, status, side_name))
        {
            return full_style;
        }

        // fallback to novel if no more specific status is available
        if matches!(
            kind,
            MatchKind::Novel { .. } | MatchKind::UnchangedPartOfNovelItem { .. } | MatchKind::NovelWord { .. }
        ) {
            if let Some(full_style) = self
                .styles
                .get(&format!("{}_novel_{}", token_kind, side_name))
            {
                return full_style;
            }
        }

        // fallback to non-novel with side name
        if matches!(
            kind,
            MatchKind::UnchangedToken { .. } | MatchKind::Ignored { .. }
        ) {
            if let Some(full_style) = self.styles.get(&format!("{}_{}", token_kind, side_name)) {
                return full_style;
            }
        }

        // fallback to side-less style
        if let Some(side_less_style) = self.styles.get(&format!("{}_{}", token_kind, status,)) {
            return side_less_style;
        }

        // fallback to the bare style for that token kind or return the base style
        if let Some(bare_style) = self.styles.get(token_kind) {
            return bare_style;
        } else {
            &self.base_style
        }

        // TODO: do we want to return the default style or is the base above enough?
        // self.default_style(novel, side)
    }
}

fn insert_style_combos(
    styles: &mut StyleMap,
    name: &str,
    style: Style,
    lhs_novel_color: yansi::Color,
    rhs_novel_color: yansi::Color,
) {
    styles.insert(
        format!("{}_novel_left", name),
        style.clone().bg(lhs_novel_color),
    );
    styles.insert(
        format!("{}_novel_right", name),
        style.clone().bg(rhs_novel_color),
    );
    styles.insert(name.to_owned(), style);
}

impl Default for Theme {
    /// Minimal highlighting inspired by the Alabaster philosophy:
    /// only strings, comments, functions, and variables get color.
    /// Keywords, types, and delimiters use the default foreground.
    /// Foreground colors use ANSI palette indices to match the terminal theme.
    fn default() -> Self {
        // default background colors
        let novel_bg_left = Color::Rgb(55, 32, 34);
        let novel_bg_right = Color::Rgb(0x1b, 0x2c, 0x1f); // 1b2c1f
        // default background highlight for sub-line differences
        let lhs_novel_color = Color::Rgb(95, 50, 54);
        let rhs_novel_color = Color::Rgb(0x1b, 0x4d, 0x28); //1b4d28

        let mut styles = HashMap::new();

        // Normal tokens (no foreground color, but need word-level novel backgrounds)
        insert_style_combos(&mut styles, "normal", yansi::Style::default(), lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "keyword", yansi::Style::default(), lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "type", yansi::Style::default(), lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "variable", Style::new().bright_blue(), lhs_novel_color, rhs_novel_color);

        // Strings — color2 (green)
        insert_style_combos(&mut styles, "string_literal", Style::new().bright_yellow(), lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "text", Style::new().yellow(), lhs_novel_color, rhs_novel_color);

        // Constants (numbers, booleans, symbols) — color5 (purple)
        insert_style_combos(&mut styles, "constant", Style::new().bright_yellow(), lhs_novel_color, rhs_novel_color);

        // Comments — color3 (yellow), warm and prominent
        insert_style_combos(&mut styles, "comment", Style::new().magenta(), lhs_novel_color, rhs_novel_color);

        // Top-level definitions — color12 (light blue)
        insert_style_combos(&mut styles, "function", Style::new().fixed(220), lhs_novel_color, rhs_novel_color);

        // Delimiters — color7 (slightly brighter gray)
        insert_style_combos(&mut styles, "delimiter", Style::new().fixed(8), lhs_novel_color, rhs_novel_color);

        // Errors — color1 (red)
        styles.insert("tree_sitter_error".to_string(), Style::new().red());

        Theme {
            novel_bg_left,
            novel_bg_right,
            base_style: yansi::Style::default(),
            novel_style_left: Style::new().bg(lhs_novel_color),
            novel_style_right: Style::new().bg(rhs_novel_color),
            lineno_style_base: Style::new().fixed(8),
            lineno_style_left: Style::new().red().bold(),
            lineno_style_right: Style::new().green().bold(),
            styles,
        }
    }
}
