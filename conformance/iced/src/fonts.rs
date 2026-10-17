//! The fonts of the snapshots: only the bundled ones, so that no glyph comes
//! from the fonts of the system and the snapshots are identical on every
//! machine.
use iced_test::renderer::graphics::text::cosmic_text::{self, Fallback, fontdb};
use unicode_script::Script;

/// The emoji font, also loaded by the shadcn/ui side (`src/index.css`).
const EMOJI: &[u8] = include_bytes!("../../fonts/NotoEmoji-Regular.ttf");

/// The fallbacks of glyphs that the requested font lacks, in the order of
/// `--font-sans` and `--font-mono` in `src/index.css`.
const FALLBACKS: &[&str] = &[shadcn_iced::font::SYMBOLS, "Noto Emoji"];

/// Replaces the font system of the renderer with one that only has the
/// bundled fonts (and the icons of iced) and falls back to [`FALLBACKS`].
pub fn load() {
    let mut font_system = iced_test::renderer::graphics::text::font_system()
        .write()
        .expect("font system");

    for font in shadcn_iced::font::all() {
        font_system.load_font(font);
    }
    font_system.load_font(EMOJI.into());

    let raw = font_system.raw();
    let (_, mut db) = std::mem::replace(
        raw,
        cosmic_text::FontSystem::new_with_locale_and_db(String::new(), fontdb::Database::new()),
    )
    .into_locale_and_db();

    let system: Vec<_> = db
        .faces()
        .filter(|face| !matches!(face.source, fontdb::Source::Binary(_)))
        .map(|face| face.id)
        .collect();

    for id in system {
        db.remove_face(id);
    }

    // The fallbacks only have a regular weight, but cosmic-text only falls
    // back to faces of the requested weight: register them at every weight,
    // like browsers use them for any weight (without synthesizing bold, see
    // `font-synthesis-weight` in `src/index.css`).
    let fallbacks: Vec<_> = db
        .faces()
        .filter(|face| {
            face.families
                .iter()
                .any(|(family, _)| FALLBACKS.contains(&family.as_str()))
        })
        .cloned()
        .collect();

    for face in fallbacks {
        for weight in (100..=900).step_by(100) {
            if weight != face.weight.0 {
                db.push_face_info(fontdb::FaceInfo {
                    weight: fontdb::Weight(weight),
                    ..face.clone()
                });
            }
        }
    }

    *raw = cosmic_text::FontSystem::new_with_locale_and_db_and_fallback(
        // The locale of the shadcn/ui side.
        "en-US".to_owned(),
        db,
        Bundled,
    );
}

struct Bundled;

impl Fallback for Bundled {
    fn common_fallback(&self) -> &[&'static str] {
        FALLBACKS
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn script_fallback(&self, _script: Script, _locale: &str) -> &[&'static str] {
        &[]
    }
}
