use anyhow::{Context, Result};
use icu_locale_core::Locale;

/// A language the pages speak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    /// Russian.
    Ru,
}

/// One text written in each language, in the order of [`Lang::ALL`].
pub type Text = [&'static str; 2];

impl Lang {
    /// The languages, in the order the language page lists them.
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Ru];

    /// The language's name, written in it.
    pub fn name(self) -> &'static str {
        self.pick(["English", "Русский"])
    }

    /// The language as a locale, which [`Lang::for_locale`] reads back.
    pub fn tag(self) -> &'static str {
        self.pick(["en", "ru"])
    }

    /// Of the same thing written in each language, in the order of
    /// [`Lang::ALL`], the one in this language.
    pub fn pick<T>(self, [en, ru]: [T; 2]) -> T {
        match self {
            Lang::En => en,
            Lang::Ru => ru,
        }
    }

    /// The language for a locale: Russian for a system built or asking in
    /// Russian, and English for every other language.
    pub fn for_locale(tag: &str) -> Result<Lang> {
        // ICU takes a hyphen alone between the parts of a locale.
        let locale = Locale::try_from_str(&tag.replace('_', "-")).with_context(|| format!("read the locale {tag:?}"))?;
        Ok(if locale.id.language.as_str() == "ru" { Lang::Ru } else { Lang::En })
    }
}

/// A text written in each language, made as [`format!`] makes it of the
/// values it names:
/// `tr!(l, "Couldn't mount {base}", "Не удалось смонтировать {base}")`.
#[macro_export]
macro_rules! tr {
    ($l:expr, $en:literal, $ru:literal $(,)?) => {
        match $l {
            $crate::lang::Lang::En => format!($en),
            $crate::lang::Lang::Ru => format!($ru),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::{TokenStream, TokenTree};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;
    use syn::visit::Visit;

    /// The texts a source file writes: its string literals, those among a
    /// macro's arguments too.
    #[derive(Default)]
    struct Texts(Vec<String>);

    impl Texts {
        fn tokens(&mut self, tokens: TokenStream) {
            for tree in tokens {
                match tree {
                    TokenTree::Literal(lit) => {
                        if let syn::Lit::Str(s) = syn::Lit::new(lit) {
                            self.0.push(s.value());
                        }
                    }
                    TokenTree::Group(group) => self.tokens(group.stream()),
                    TokenTree::Ident(_) | TokenTree::Punct(_) => {}
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Texts {
        fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
            self.0.push(lit.value());
        }

        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            self.tokens(mac.tokens.clone());
        }
    }

    #[test]
    fn the_typeface_draws_every_text_the_pages_write() {
        let f = crate::font::bundled();
        let mut texts = Texts::default();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in fs::read_dir(&src).unwrap() {
            let path = entry.unwrap().path();
            let file = syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
            texts.visit_file(&file);
        }
        assert!(texts.0.iter().any(|t| t.contains("Расшифровка")), "the texts in Russian are read");
        let missing: BTreeSet<char> = texts.0.iter().flat_map(|t| t.chars()).filter(|&ch| !ch.is_control() && !f.body.has(ch)).collect();
        assert!(missing.is_empty(), "MiSans draws none of {missing:?}");
    }

    #[test]
    fn a_locale_picks_the_language() {
        let of = |tag| Lang::for_locale(tag).unwrap();
        assert_eq!(of("ru-RU"), Lang::Ru);
        assert_eq!(of("ru_RU"), Lang::Ru, "as older systems write it");
        assert_eq!(of("ru"), Lang::Ru);
        assert_eq!(of("en-US"), Lang::En);
        assert_eq!(of("zh-CN"), Lang::En, "a language the pages do not speak");
        assert_eq!(of("ja-JP"), Lang::En);
        assert!(Lang::for_locale("not a locale").is_err());
    }

    #[test]
    fn each_language_s_locale_reads_back_as_it() {
        for lang in Lang::ALL {
            assert_eq!(Lang::for_locale(lang.tag()).unwrap(), lang);
        }
    }

    #[test]
    fn a_text_is_picked_and_made_in_the_language() {
        assert_eq!(Lang::Ru.pick(["Settings", "Настройки"]), "Настройки");
        let base = "vendor";
        assert_eq!(tr!(Lang::En, "Couldn't mount {base}", "Не удалось смонтировать {base}"), "Couldn't mount vendor");
        assert_eq!(tr!(Lang::Ru, "Couldn't mount {base}", "Не удалось смонтировать {base}"), "Не удалось смонтировать vendor");
        assert_eq!(Lang::ALL.map(Lang::name), ["English", "Русский"]);
    }
}
