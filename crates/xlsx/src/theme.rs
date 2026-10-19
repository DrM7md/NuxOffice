//! `theme1.xml`: colour scheme and font scheme.
//!
//! In the file the colour scheme is ordered dk1, lt1, dk2, lt2, accent1–6, hlink, folHlink,
//! while cell colour `theme` indices (and the model's `Theme.colors`) use lt1, dk1, lt2, dk2, ….

use std::fmt::Write as _;

use gridcraft_model::{Theme, ThemeFonts};

use crate::xml::{El, esc_attr};

const SLOTS: [(&str, usize); 12] = [
    ("dk1", 1),
    ("lt1", 0),
    ("dk2", 3),
    ("lt2", 2),
    ("accent1", 4),
    ("accent2", 5),
    ("accent3", 6),
    ("accent4", 7),
    ("accent5", 8),
    ("accent6", 9),
    ("hlink", 10),
    ("folHlink", 11),
];
const MAX_SCRIPT_FONTS: usize = 64;
const MAX_TYPEFACE_CHARS: usize = 256;

fn scheme_rgb(e: &El) -> Option<u32> {
    if let Some(c) = e.child("srgbClr").and_then(|c| c.attr("val")) {
        return u32::from_str_radix(c.trim(), 16).ok().map(|v| v & 0xFFFFFF);
    }
    if let Some(c) = e.child("sysClr") {
        if let Some(v) = c.attr("lastClr").and_then(|v| u32::from_str_radix(v.trim(), 16).ok()) {
            return Some(v & 0xFFFFFF);
        }
        return match c.attr("val") {
            Some("windowText") => Some(0x000000),
            Some("window") => Some(0xFFFFFF),
            _ => None,
        };
    }
    None
}

pub fn read_theme(root: &El) -> Theme {
    let mut t = Theme::default();
    if let Some(n) = root.attr("name") {
        t.name = n.to_string();
    }
    let Some(el) = root.child("themeElements") else { return t };
    if let Some(cs) = el.child("clrScheme") {
        for (name, idx) in SLOTS {
            if let Some(v) = cs.child(name).and_then(scheme_rgb)
                && let Some(slot) = t.colors.get_mut(idx)
            {
                *slot = v;
            }
        }
    }
    if let Some(fs) = el.child("fontScheme") {
        if let Some(major) = fs.child("majorFont") {
            read_font_collection(major, &mut t.major_font, &mut t.major_fonts);
        }
        if let Some(minor) = fs.child("minorFont") {
            read_font_collection(minor, &mut t.minor_font, &mut t.minor_fonts);
        }
    }
    t
}

/// A minimal, schema-complete theme part for `t`.
pub fn write_theme(t: &Theme) -> String {
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    let name = if t.name.is_empty() { "Craft" } else { &t.name };
    let _ = write!(s, "<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"{}\"><a:themeElements>", esc_attr(name));
    let _ = write!(s, "<a:clrScheme name=\"{}\">", esc_attr(name));
    for (slot, idx) in SLOTS {
        let rgb = t.colors.get(idx).copied().unwrap_or(0) & 0xFFFFFF;
        let _ = write!(s, "<a:{slot}><a:srgbClr val=\"{rgb:06X}\"/></a:{slot}>");
    }
    s.push_str("</a:clrScheme>");
    let _ = write!(s, "<a:fontScheme name=\"{}\">", esc_attr(name));
    write_font_collection(&mut s, "majorFont", &t.major_font, &t.major_fonts);
    write_font_collection(&mut s, "minorFont", &t.minor_font, &t.minor_fonts);
    s.push_str("</a:fontScheme>");
    // Format scheme: three entries per list, as the schema requires.
    s.push_str("<a:fmtScheme name=\"Craft\"><a:fillStyleLst>");
    for _ in 0..3 {
        s.push_str("<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>");
    }
    s.push_str("</a:fillStyleLst><a:lnStyleLst>");
    for w in [6350, 12700, 19050] {
        let _ = write!(
            s,
            "<a:ln w=\"{w}\" cap=\"flat\" cmpd=\"sng\" algn=\"ctr\"><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill><a:prstDash val=\"solid\"/></a:ln>"
        );
    }
    s.push_str("</a:lnStyleLst><a:effectStyleLst>");
    for _ in 0..3 {
        s.push_str("<a:effectStyle><a:effectLst/></a:effectStyle>");
    }
    s.push_str("</a:effectStyleLst><a:bgFillStyleLst>");
    for _ in 0..3 {
        s.push_str("<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>");
    }
    s.push_str("</a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>");
    s
}

fn typeface(el: &El, child: &str) -> Option<String> {
    el.child(child).and_then(|e| e.attr("typeface")).map(|font| font.chars().take(MAX_TYPEFACE_CHARS).collect())
}

fn read_font_collection(el: &El, latin: &mut String, fonts: &mut ThemeFonts) {
    if let Some(face) = typeface(el, "latin").filter(|face| !face.is_empty()) {
        *latin = face;
    }
    fonts.ea = typeface(el, "ea").unwrap_or_default();
    fonts.cs = typeface(el, "cs").unwrap_or_default();
    fonts.scripts = el
        .children
        .iter()
        .filter(|child| child.name == "font")
        .take(MAX_SCRIPT_FONTS)
        .map(|child| {
            (
                child.attr("script").unwrap_or_default().to_string(),
                child.attr("typeface").unwrap_or_default().chars().take(MAX_TYPEFACE_CHARS).collect(),
            )
        })
        .collect();
}

fn write_font_collection(s: &mut String, name: &str, latin: &str, fonts: &ThemeFonts) {
    let _ = write!(
        s,
        "<a:{name}><a:latin typeface=\"{}\"/><a:ea typeface=\"{}\"/><a:cs typeface=\"{}\"/>",
        esc_attr(latin),
        esc_attr(&fonts.ea),
        esc_attr(&fonts.cs)
    );
    for (script, typeface) in &fonts.scripts {
        let _ = write!(s, "<a:font script=\"{}\" typeface=\"{}\"/>", esc_attr(script), esc_attr(typeface));
    }
    let _ = write!(s, "</a:{name}>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut t = Theme::default();
        t.colors[0] = 0xFEFEFE;
        t.colors[1] = 0x101010;
        t.colors[9] = 0x123456;
        t.major_font = "Cambria".into();
        t.major_fonts.cs = "Arial".into();
        t.minor_fonts.scripts = vec![("Arab".into(), "Times New Roman".into())];
        t.name = "Mine".into();
        let x = write_theme(&t);
        let back = read_theme(&crate::xml::parse(x.as_bytes()).unwrap());
        assert_eq!(back, t);
    }

    #[test]
    fn theme_keeps_complex_script_and_per_script_fonts() {
        let source = r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Fonts"><a:themeElements><a:fontScheme name="Fonts"><a:majorFont><a:latin typeface="Major"/><a:ea typeface="Major EA"/><a:cs typeface="Arial"/><a:font script="Arab" typeface="Times New Roman"/><a:font script="Hebr" typeface="Hebrew"/><a:font script="Jpan" typeface="Japanese"/></a:majorFont><a:minorFont><a:latin typeface="Minor"/><a:ea typeface="Minor EA"/><a:cs typeface="Arial"/><a:font script="Arab" typeface="Times New Roman"/><a:font script="Hebr" typeface="Hebrew"/><a:font script="Jpan" typeface="Japanese"/></a:minorFont></a:fontScheme></a:themeElements></a:theme>"#;
        let theme = read_theme(&crate::xml::parse(source.as_bytes()).unwrap());
        let output = write_theme(&theme);
        let reopened = read_theme(&crate::xml::parse(output.as_bytes()).unwrap());
        let output = crate::xml::parse(output.as_bytes()).unwrap();
        let fonts = output.path(&["themeElements", "fontScheme"]);
        for (collection, ea) in [("majorFont", "Major EA"), ("minorFont", "Minor EA")] {
            let collection = fonts.and_then(|scheme| scheme.child(collection));
            assert_eq!(collection.and_then(|e| e.path(&["ea"]).and_then(|e| e.attr("typeface"))), Some(ea));
            assert_eq!(collection.and_then(|e| e.path(&["cs"]).and_then(|e| e.attr("typeface"))), Some("Arial"));
            let scripts: Vec<_> = collection
                .into_iter()
                .flat_map(|e| e.children.iter())
                .filter(|e| e.name == "font")
                .map(|e| (e.attr("script").unwrap_or_default(), e.attr("typeface").unwrap_or_default()))
                .collect();
            assert_eq!(scripts, [("Arab", "Times New Roman"), ("Hebr", "Hebrew"), ("Jpan", "Japanese")]);
        }
        assert_eq!(reopened.major_font, "Major");
        assert_eq!(reopened.minor_font, "Minor");
    }

    #[test]
    fn default_theme_writes_unchanged_font_scheme() {
        let output = write_theme(&Theme::default());
        assert!(output.contains("<a:majorFont><a:latin typeface=\"Calibri\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:majorFont><a:minorFont><a:latin typeface=\"Calibri\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:minorFont>"));
    }

    #[test]
    fn theme_caps_hostile_script_font_lists() {
        let entries = (0..100).map(|i| format!("<a:font script=\"S{i}\" typeface=\"{}\"/>", "x".repeat(300))).collect::<String>();
        let source = format!(
            "<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:themeElements><a:fontScheme><a:majorFont><a:latin typeface=\"Major\"/>{entries}</a:majorFont></a:fontScheme></a:themeElements></a:theme>"
        );
        let theme = read_theme(&crate::xml::parse(source.as_bytes()).unwrap());
        let output = write_theme(&theme);
        let output = crate::xml::parse(output.as_bytes()).unwrap();
        let fonts = output.path(&["themeElements", "fontScheme", "majorFont"]);
        let scripts: Vec<_> = fonts.into_iter().flat_map(|e| e.children.iter()).filter(|e| e.name == "font").collect();
        assert_eq!(scripts.len(), 64);
        assert!(scripts.iter().all(|e| e.attr("typeface").is_some_and(|f| f.chars().count() <= 256)));
    }
}
