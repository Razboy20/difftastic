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

// --- Oklab color blending ---

struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn rgb_to_oklab(r: u8, g: u8, b: u8) -> Oklab {
    let r = srgb_to_linear(r as f32 / 255.0);
    let g = srgb_to_linear(g as f32 / 255.0);
    let b = srgb_to_linear(b as f32 / 255.0);

    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.7136952004 * g + 0.0743913015 * b).cbrt();
    let s = (0.0883024619 * r + 0.2289690106 * g + 0.6827285272 * b).cbrt();

    Oklab {
        l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    }
}

fn oklab_to_rgb(lab: &Oklab) -> (u8, u8, u8) {
    let l = lab.l + 0.3963377774 * lab.a + 0.2158037573 * lab.b;
    let m = lab.l - 0.1055613458 * lab.a - 0.0638541728 * lab.b;
    let s = lab.l - 0.0894841775 * lab.a - 1.2914855480 * lab.b;

    let l = l * l * l;
    let m = m * m * m;
    let s = s * s * s;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (
        (linear_to_srgb(r.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8,
        (linear_to_srgb(g.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8,
        (linear_to_srgb(b.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8,
    )
}

fn blend_oklab(c1: (u8, u8, u8), c2: (u8, u8, u8), t: f32) -> Color {
    let a = rgb_to_oklab(c1.0, c1.1, c1.2);
    let b = rgb_to_oklab(c2.0, c2.1, c2.2);
    let mixed = Oklab {
        l: a.l * (1.0 - t) + b.l * t,
        a: a.a * (1.0 - t) + b.a * t,
        b: a.b * (1.0 - t) + b.b * t,
    };
    let (r, g, b) = oklab_to_rgb(&mixed);
    Color::Rgb(r, g, b)
}

/// Compute a blend factor based on Oklab distance from white.
/// White (unstyled text) gets fully replaced with the red/green target.
/// The more chromatic or dark a color is, the less it gets blended,
/// preserving its syntax meaning while adding a subtle tint.
fn oklab_dist_from_white(fg: (u8, u8, u8)) -> f32 {
    const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
    let src = rgb_to_oklab(fg.0, fg.1, fg.2);
    let wht = rgb_to_oklab(WHITE.0, WHITE.1, WHITE.2);
    let dl = src.l - wht.l;
    let da = src.a - wht.a;
    let db = src.b - wht.b;
    (dl * dl + da * da + db * db).sqrt()
}

fn adaptive_blend_factor(fg: (u8, u8, u8), min_blend: f32) -> f32 {
    let dist = oklab_dist_from_white(fg);
    // The linear segment endpoints and quadratic rise target
    // all shift proportionally with min_blend.
    // min_blend=0.4 → linear 1.0→0.4, quad 0.4→0.8
    // min_blend=0.6 → linear 1.0→0.6, quad 0.6→0.8
    let peak = 0.8_f32.max(min_blend);

    if dist <= 0.1 {
        // Linear: 1.0 at dist=0, min_blend at dist=0.1
        1.0 + (min_blend - 1.0) * (dist / 0.1)
    } else {
        // Quadratic: min_blend at dist=0.1, peak at dist=0.5
        // f(x) = a(x-0.1)² + min_blend, where f(0.5) = peak
        // peak = a·0.16 + min_blend → a = (peak - min_blend) / 0.16
        let a = (peak - min_blend) / 0.16;
        let dx = dist - 0.1;
        (a * dx * dx + min_blend).min(1.0)
    }
}

// --- Style combos with Oklab-blended novel foregrounds ---

fn insert_style_combos(
    styles: &mut StyleMap,
    name: &str,
    fg_rgb: (u8, u8, u8),
    del_target: (u8, u8, u8),
    add_target: (u8, u8, u8),
    lhs_novel_bg: yansi::Color,
    rhs_novel_bg: yansi::Color,
) {
    let base = Style::new().fg(Color::Rgb(fg_rgb.0, fg_rgb.1, fg_rgb.2));
    let left_fg = blend_oklab(fg_rgb, del_target, adaptive_blend_factor(fg_rgb, 0.6));
    let right_fg = blend_oklab(fg_rgb, add_target, adaptive_blend_factor(fg_rgb, 0.4));

    styles.insert(
        format!("{}_novel_left", name),
        Style::new().fg(left_fg).bg(lhs_novel_bg),
    );
    styles.insert(
        format!("{}_novel_right", name),
        Style::new().fg(right_fg).bg(rhs_novel_bg),
    );
    styles.insert(name.to_owned(), base);
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

        // Terminal palette RGB values for blending
        let fg          = (0xff, 0xff, 0xff); // foreground
        let bright_yel  = (0xe0, 0xbe, 0x9f); // color11
        let yellow      = (0xdb, 0xa8, 0x78); // color3
        let magenta     = (0xd1, 0x90, 0xe4); // color5
        let bright_blue = (0x92, 0xd8, 0xfc); // color12
        let dark_gray   = (0x63, 0x6e, 0x7f); // color8
        let gold        = (0xff, 0xd7, 0x00); // fixed(220)

        // Blend targets: deleted → bright red, added → bright green
        let del = (0xf0, 0xa7, 0xab); // color9  #f0a7ab
        let add = (0xc7, 0xdf, 0xae); // color10 #c7dfae

        // Normal tokens (default fg)
        insert_style_combos(&mut styles, "normal", fg, del, add, lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "keyword", fg, del, add, lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "type", fg, del, add, lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "variable", bright_blue, del, add, lhs_novel_color, rhs_novel_color);

        // Strings
        insert_style_combos(&mut styles, "string_literal", bright_yel, del, add, lhs_novel_color, rhs_novel_color);
        insert_style_combos(&mut styles, "text", yellow, del, add, lhs_novel_color, rhs_novel_color);

        // Constants
        insert_style_combos(&mut styles, "constant", bright_yel, del, add, lhs_novel_color, rhs_novel_color);

        // Comments
        insert_style_combos(&mut styles, "comment", magenta, del, add, lhs_novel_color, rhs_novel_color);

        // Top-level definitions
        insert_style_combos(&mut styles, "function", gold, del, add, lhs_novel_color, rhs_novel_color);

        // Delimiters
        insert_style_combos(&mut styles, "delimiter", dark_gray, del, add, lhs_novel_color, rhs_novel_color);

        // Errors
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
