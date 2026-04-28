//! Public API for LSP tooling: enumerate all built-in trigger, effect, and
//! iterator names for the compiled game (determined by feature flag).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspEntryKind {
    Trigger,
    Effect,
    /// Base name — represents every_X, any_X, random_X, ordered_X variants.
    Iterator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspEntry {
    pub name: String,
    pub kind: LspEntryKind,
}

/// Return all built-in trigger, effect, and iterator names for the compiled game.
pub fn all_builtin_entries() -> Vec<LspEntry> {
    let mut out = Vec::new();

    fn push(
        out: &mut Vec<LspEntry>,
        triggers: Vec<&'static str>,
        effects: Vec<&'static str>,
        iterators: Vec<&'static str>,
    ) {
        for name in triggers {
            out.push(LspEntry { name: name.to_owned(), kind: LspEntryKind::Trigger });
        }
        for name in effects {
            out.push(LspEntry { name: name.to_owned(), kind: LspEntryKind::Effect });
        }
        for base in iterators {
            for prefix in &["every_", "any_", "random_", "ordered_"] {
                out.push(LspEntry {
                    name: format!("{prefix}{base}"),
                    kind: LspEntryKind::Iterator,
                });
            }
        }
    }

    #[cfg(feature = "vic3")]
    push(
        &mut out,
        crate::vic3::tables::trigger_names(),
        crate::vic3::tables::effect_names(),
        crate::vic3::tables::iterator_names(),
    );

    #[cfg(feature = "ck3")]
    push(
        &mut out,
        crate::ck3::tables::trigger_names(),
        crate::ck3::tables::effect_names(),
        crate::ck3::tables::iterator_names(),
    );

    #[cfg(feature = "imperator")]
    push(
        &mut out,
        crate::imperator::tables::trigger_names(),
        crate::imperator::tables::effect_names(),
        crate::imperator::tables::iterator_names(),
    );

    #[cfg(feature = "hoi4")]
    push(
        &mut out,
        crate::hoi4::tables::trigger_names(),
        crate::hoi4::tables::effect_names(),
        crate::hoi4::tables::iterator_names(),
    );

    #[cfg(feature = "eu5")]
    push(
        &mut out,
        crate::eu5::tables::trigger_names(),
        crate::eu5::tables::effect_names(),
        crate::eu5::tables::iterator_names(),
    );

    out
}
