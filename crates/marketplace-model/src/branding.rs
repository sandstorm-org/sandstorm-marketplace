//! Public, per-marketplace appearance settings and deterministic header colors.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const DEFAULT_COLOR: &str = "#222222";
const PALETTE_REFERENCE_COLOR: &str = "#552976";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Branding {
    pub title: String,
    pub header_color: String,
    #[serde(default)]
    pub show_subheader: bool,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            title: "App Market".into(),
            header_color: DEFAULT_COLOR.into(),
            show_subheader: false,
        }
    }
}

impl Branding {
    pub fn validate(&mut self) -> Result<()> {
        self.title = self.title.trim().to_owned();
        ensure!(
            !self.title.is_empty()
                && self.title.chars().count() <= 80
                && !self.title.chars().any(char::is_control),
            "title must be 1–80 characters without control characters"
        );
        ensure!(
            rgb(&self.header_color).is_some(),
            "header color must be a six-digit hex color, such as #552976"
        );
        self.header_color.make_ascii_lowercase();
        Ok(())
    }

    pub fn load(root: &Path) -> Result<Self> {
        let mut settings: Self = match fs::read(root.join("branding.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error.into()),
        };
        settings.validate()?;
        Ok(settings)
    }

    /// Preserve the original purple palette exactly, shifting hue and scaling
    /// saturation/lightness relative to its base for other chosen colors.
    pub fn header_style(&self) -> String {
        let base = hsl(rgb(&self.header_color).unwrap_or_else(|| rgb(DEFAULT_COLOR).unwrap()));
        let original = hsl(rgb(PALETTE_REFERENCE_COLOR).unwrap());
        let colors: Vec<_> = [
            "#723086", "#552976", "#603281", "#4e2e77", "#7e3d92", "#924da7", "#a170af",
        ]
        .into_iter()
        .map(|color| {
            let reference = hsl(rgb(color).unwrap());
            from_hsl([
                (base[0] + reference[0] - original[0]).rem_euclid(360.0),
                (base[1] * reference[1] / original[1]).clamp(0.0, 1.0),
                (base[2] * reference[2] / original[2]).clamp(0.0, 1.0),
            ])
        })
        .collect();
        let hex: Vec<_> = colors
            .iter()
            .map(|c| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]))
            .collect();
        let text = foreground(&colors[..4]);
        format!(
            "--header-gradient:linear-gradient(165deg, {}, {} 60%, {} 60.1%, {});--header-text:{};--header-invert:{};--header-control:{};--header-control-hover:{};--header-control-text:{};--header-search:{};--header-search-text:{}",
            hex[0],
            hex[1],
            hex[2],
            hex[3],
            text,
            if text == "#ffffff" { 1 } else { 0 },
            hex[4],
            hex[5],
            foreground(&colors[4..6]),
            hex[6],
            foreground(&colors[6..])
        )
    }
}

fn rgb(hex: &str) -> Option<[u8; 3]> {
    if hex.len() != 7
        || !hex.starts_with('#')
        || !hex.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
    {
        return None;
    }
    Some([
        u8::from_str_radix(&hex[1..3], 16).ok()?,
        u8::from_str_radix(&hex[3..5], 16).ok()?,
        u8::from_str_radix(&hex[5..7], 16).ok()?,
    ])
}

fn hsl(rgb: [u8; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(|c| c as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let l = (max + min) / 2.0;
    if delta == 0.0 {
        return [0.0, 0.0, l];
    }
    let h = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    [h * 60.0, delta / (1.0 - (2.0 * l - 1.0).abs()), l]
}

fn from_hsl([h, s, l]: [f64; 3]) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let values = match (h / 60.0) as u8 {
        0 => [c, x, 0.0],
        1 => [x, c, 0.0],
        2 => [0.0, c, x],
        3 => [0.0, x, c],
        4 => [x, 0.0, c],
        _ => [c, 0.0, x],
    };
    values.map(|v| ((v + l - c / 2.0) * 255.0).round().clamp(0.0, 255.0) as u8)
}

fn foreground(colors: &[[u8; 3]]) -> &'static str {
    let mut white: f64 = 21.0;
    let mut black: f64 = 21.0;
    for color in colors {
        let [r, g, b] = color.map(|c| {
            let v = c as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        });
        let l = r * 0.2126 + g * 0.7152 + b * 0.0722;
        white = white.min(1.05 / (l + 0.05));
        black = black.min((l + 0.05) / 0.05);
    }
    if white >= black { "#ffffff" } else { "#000000" }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn purple_still_reproduces_original_gradient() {
        let branding = Branding {
            header_color: PALETTE_REFERENCE_COLOR.into(),
            ..Branding::default()
        };
        assert!(branding.header_style().starts_with("--header-gradient:linear-gradient(165deg, #723086, #552976 60%, #603281 60.1%, #4e2e77);"));
    }
    #[test]
    fn default_header_is_charcoal() {
        let branding = Branding::default();
        assert_eq!(branding.header_color, "#222222");
        assert!(branding.header_style().starts_with("--header-gradient:linear-gradient(165deg, #272727, #222222 60%, #262626 60.1%, #232323);"));
    }
    #[test]
    fn existing_settings_default_to_no_subheader() {
        let settings: Branding =
            serde_json::from_str(r##"{"title":"Existing Market","headerColor":"#57575f"}"##)
                .unwrap();
        assert!(!settings.show_subheader);
        assert_eq!(settings.title, "Existing Market");
        assert_eq!(settings.header_color, "#57575f");
    }
    #[test]
    fn validates_and_handles_extreme_colors() {
        for (color, text) in [("#000000", "#ffffff"), ("#ffffff", "#000000")] {
            let mut b = Branding {
                title: " My Market ".into(),
                header_color: color.into(),
                show_subheader: false,
            };
            b.validate().unwrap();
            assert_eq!(b.title, "My Market");
            assert!(b.header_style().contains(&format!("--header-text:{text};")));
        }
        for color in ["red", "#fff", "#123456;", "#éabcd"] {
            assert!(
                Branding {
                    header_color: color.into(),
                    ..Branding::default()
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            Branding {
                title: " ".into(),
                ..Branding::default()
            }
            .validate()
            .is_err()
        );
    }
}
