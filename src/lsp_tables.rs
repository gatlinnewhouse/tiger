//! Public API for LSP tooling: enumerate all built-in trigger, effect, and
//! iterator names for the compiled game (determined by feature flag).
//!
//! Also provides schema extraction for signature-help: `block_schema(name)`
//! returns the fields of any trigger/effect that takes a block argument.

use crate::effect::Effect;
use crate::trigger::Trigger;

// ─── Schema types ─────────────────────────────────────────────────────────────

/// One field inside a block-style trigger or effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    /// Field name as it appears in script (no leading `?` or `+`).
    pub name: String,
    /// `false` when the original name had a `?` prefix (optional field).
    pub required: bool,
    /// Human-readable type hint, e.g. "value", "bool", "scope:Character".
    pub type_hint: String,
}

/// Return the schema for a built-in trigger or effect named `name`, if it
/// accepts a block argument.  Returns `None` for scalar triggers.
pub fn block_schema(name: &str) -> Option<Vec<SchemaField>> {
    #[cfg(feature = "vic3")]
    {
        if let result @ Some(_) = crate::vic3::tables::trigger_schema(name) {
            return result;
        }
        if let result @ Some(_) = crate::vic3::tables::effect_schema(name) {
            return result;
        }
    }
    #[cfg(feature = "ck3")]
    {
        if let result @ Some(_) = crate::ck3::tables::trigger_schema(name) {
            return result;
        }
        if let result @ Some(_) = crate::ck3::tables::effect_schema(name) {
            return result;
        }
    }
    #[cfg(feature = "imperator")]
    {
        if let result @ Some(_) = crate::imperator::tables::trigger_schema(name) {
            return result;
        }
        if let result @ Some(_) = crate::imperator::tables::effect_schema(name) {
            return result;
        }
    }
    #[cfg(feature = "hoi4")]
    {
        if let result @ Some(_) = crate::hoi4::tables::trigger_schema(name) {
            return result;
        }
        if let result @ Some(_) = crate::hoi4::tables::effect_schema(name) {
            return result;
        }
    }
    #[cfg(feature = "eu5")]
    {
        if let result @ Some(_) = crate::eu5::tables::trigger_schema(name) {
            return result;
        }
        if let result @ Some(_) = crate::eu5::tables::effect_schema(name) {
            return result;
        }
    }
    None
}

/// Extract `SchemaField`s from a block field array (as stored in `Trigger::Block`).
/// Field names starting with `?` are marked optional; other prefixes are stripped.
pub(crate) fn extract_block_fields(fields: &[(&'static str, Trigger)]) -> Vec<SchemaField> {
    fields
        .iter()
        .map(|(raw_name, trigger)| {
            let (name, required) = if let Some(n) = raw_name.strip_prefix('?') {
                (n.to_owned(), false)
            } else if let Some(n) = raw_name.strip_prefix('+') {
                // `+field` means "required, repeatable" in some tables
                (n.to_owned(), true)
            } else {
                ((*raw_name).to_owned(), true)
            };
            SchemaField { name, required, type_hint: trigger_to_hint(trigger) }
        })
        .collect()
}

/// Extract schema fields from an `Effect` variant for block-style effects.
/// Returns `None` for scalar effects, `Vb/Vbc/Vbv`-validated effects (no static schema),
/// and control effects (open-ended blocks).
pub(crate) fn effect_to_schema(e: &Effect) -> Option<Vec<SchemaField>> {
    let field = |name: &str, hint: &str| SchemaField {
        name: name.to_owned(),
        required: true,
        type_hint: hint.to_owned(),
    };
    match e {
        #[cfg(feature = "ck3")]
        Effect::Target(fname, scope) => Some(vec![field(fname, &format!("scope:{scope:?}"))]),
        #[cfg(any(feature = "ck3", feature = "vic3"))]
        Effect::TargetValue(tf, scope, vf) => Some(vec![
            field(tf, &format!("scope:{scope:?}")),
            field(vf, "value"),
        ]),
        #[cfg(any(feature = "ck3", feature = "hoi4"))]
        Effect::ItemTarget(if_, item, sf, scope) => Some(vec![
            field(if_, &format!("item:{item:?}")),
            field(sf, &format!("scope:{scope:?}")),
        ]),
        #[cfg(feature = "ck3")]
        Effect::ItemValue(if_, item) => Some(vec![field(if_, &format!("item:{item:?}"))]),
        #[cfg(any(feature = "ck3", feature = "vic3"))]
        Effect::Timespan => Some(vec![
            SchemaField { name: "days".to_owned(),   required: false, type_hint: "value".to_owned() },
            SchemaField { name: "weeks".to_owned(),  required: false, type_hint: "value".to_owned() },
            SchemaField { name: "months".to_owned(), required: false, type_hint: "value".to_owned() },
            SchemaField { name: "years".to_owned(),  required: false, type_hint: "value".to_owned() },
        ]),
        #[cfg(feature = "hoi4")]
        Effect::Iterator(_, scopes) => Some(vec![SchemaField {
            name: "scope".to_owned(),
            required: false,
            type_hint: format!("{scopes:?}"),
        }]),
        // Block-validated effects with no static schema.
        // Return Some(empty) to indicate "takes a block" without exposing fields.
        Effect::Vb(_) | Effect::Vbc(_) | Effect::Vbv(_) => Some(vec![]),
        _ => None,
    }
}

/// Convert a `Trigger` variant to a short human-readable type hint string.
fn trigger_to_hint(t: &Trigger) -> String {
    match t {
        Trigger::Boolean => "yes/no".to_owned(),
        Trigger::CompareValue => "value".to_owned(),
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5", feature = "hoi4"))]
        Trigger::CompareValueWarnEq => "value".to_owned(),
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5"))]
        Trigger::SetValue => "value".to_owned(),
        Trigger::CompareDate => "date".to_owned(),
        #[cfg(any(feature = "vic3", feature = "eu5"))]
        Trigger::ItemOrCompareValue(item) => format!("item:{item:?}|value"),
        Trigger::Scope(s) | Trigger::ScopeOkThis(s) => format!("scope:{s:?}"),
        Trigger::Item(item) => format!("item:{item:?}"),
        Trigger::ScopeOrItem(s, item) => format!("scope:{s:?}|item:{item:?}"),
        Trigger::Choice(choices) => {
            let joined = choices.join("|");
            if joined.len() > 40 { format!("choice({}..)", &joined[..37]) } else { format!("choice:{joined}") }
        }
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5"))]
        Trigger::CompareChoice(choices) => {
            let joined = choices.join("|");
            if joined.len() > 40 { format!("choice({}..)", &joined[..37]) } else { format!("choice:{joined}") }
        }
        #[cfg(any(feature = "vic3", feature = "eu5"))]
        Trigger::CompareChoiceOrNumber(choices) => {
            let joined = choices.join("|");
            if joined.len() > 40 { format!("choice({}..)|value", &joined[..37]) } else { format!("choice:{joined}|value") }
        }
        Trigger::Block(_) => "block".to_owned(),
        #[cfg(feature = "ck3")]
        Trigger::ScopeOrBlock(s, _) => format!("scope:{s:?}|block"),
        #[cfg(feature = "ck3")]
        Trigger::ItemOrBlock(item, _) => format!("item:{item:?}|block"),
        #[cfg(feature = "ck3")]
        Trigger::IdentifierOrBlock(id, _) => format!("identifier:{id}|block"),
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5"))]
        Trigger::BlockOrCompareValue(_) => "block|value".to_owned(),
        #[cfg(feature = "ck3")]
        Trigger::ScopeList(s) => format!("list(scope:{s:?})"),
        #[cfg(feature = "ck3")]
        Trigger::ScopeCompare(s) => format!("scope:{s:?}"),
        #[cfg(feature = "ck3")]
        Trigger::CompareToScope(s) => format!("scope:{s:?}"),
        #[cfg(feature = "hoi4")]
        Trigger::Iterator(_, s) => format!("scope:{s:?}"),
        Trigger::Identifier(id) => format!("identifier:{id}"),
        #[cfg(feature = "hoi4")]
        Trigger::Flag => "flag".to_owned(),
        #[cfg(feature = "hoi4")]
        Trigger::FlagOrBlock(_) => "flag|block".to_owned(),
        #[cfg(any(feature = "ck3", feature = "vic3", feature = "eu5"))]
        Trigger::Removed(_, _) => "removed".to_owned(),
        Trigger::Control => "block".to_owned(),
        Trigger::Special | Trigger::UncheckedValue | Trigger::UncheckedTodo => "any".to_owned(),
    }
}

// ─── Item value lookup ────────────────────────────────────────────────────────

/// Returns the item type name (snake_case, e.g. `"building"`, `"trait"`) for the primary
/// value of a trigger/effect named `name`, if it directly takes a single `Item` value.
///
/// Used for context-sensitive completions: when the cursor is at `add_building = <cursor>`,
/// call `field_value_item("add_building")` to get `"building"` and filter completions.
pub fn field_value_item(name: &str) -> Option<&'static str> {
    #[cfg(feature = "ck3")]
    if let r @ Some(_) = crate::ck3::tables::trigger_value_item(name)
        .or_else(|| crate::ck3::tables::effect_value_item(name))
    { return r; }
    #[cfg(feature = "vic3")]
    if let r @ Some(_) = crate::vic3::tables::trigger_value_item(name)
        .or_else(|| crate::vic3::tables::effect_value_item(name))
    { return r; }
    #[cfg(feature = "imperator")]
    if let r @ Some(_) = crate::imperator::tables::trigger_value_item(name)
        .or_else(|| crate::imperator::tables::effect_value_item(name))
    { return r; }
    #[cfg(feature = "hoi4")]
    if let r @ Some(_) = crate::hoi4::tables::trigger_value_item(name)
        .or_else(|| crate::hoi4::tables::effect_value_item(name))
    { return r; }
    #[cfg(feature = "eu5")]
    if let r @ Some(_) = crate::eu5::tables::trigger_value_item(name)
        .or_else(|| crate::eu5::tables::effect_value_item(name))
    { return r; }
    None
}

/// Returns the `common/` filesystem path for items expected as the primary value of a
/// trigger/effect, e.g. `"add_building"` → `"common/buildings/"`.
///
/// More precise than `field_value_item`: uses `Item::path()` from tiger-tables, so the
/// returned string is exactly the subdirectory scanned during game loading.
pub fn field_item_path(name: &str) -> Option<&'static str> {
    #[cfg(feature = "ck3")]
    if let r @ Some(_) = crate::ck3::tables::trigger_item_path(name)
        .or_else(|| crate::ck3::tables::effect_item_path(name))
    { return r; }
    #[cfg(feature = "vic3")]
    if let r @ Some(_) = crate::vic3::tables::trigger_item_path(name)
        .or_else(|| crate::vic3::tables::effect_item_path(name))
    { return r; }
    #[cfg(feature = "imperator")]
    if let r @ Some(_) = crate::imperator::tables::trigger_item_path(name)
        .or_else(|| crate::imperator::tables::effect_item_path(name))
    { return r; }
    #[cfg(feature = "hoi4")]
    if let r @ Some(_) = crate::hoi4::tables::trigger_item_path(name)
        .or_else(|| crate::hoi4::tables::effect_item_path(name))
    { return r; }
    #[cfg(feature = "eu5")]
    if let r @ Some(_) = crate::eu5::tables::trigger_item_path(name)
        .or_else(|| crate::eu5::tables::effect_item_path(name))
    { return r; }
    None
}

/// Returns `true` if the named trigger or effect takes a flag name as its value.
/// Only relevant for HOI4 (`Trigger::Flag`, `Trigger::FlagOrBlock`).
pub fn field_is_flag(name: &str) -> bool {
    #[cfg(feature = "hoi4")]
    { if crate::hoi4::tables::triggers::trigger_is_flag(name) { return true; } }
    false
}

/// Returns the static list of valid string choices for a trigger/effect that accepts
/// a constrained set of values, e.g. `has_government = oligarchy`.
///
/// Returns `None` for triggers/effects that accept free-form or item values.
pub fn field_value_choices(name: &str) -> Option<&'static [&'static str]> {
    #[cfg(feature = "ck3")]
    if let r @ Some(_) = crate::ck3::tables::triggers::trigger_value_choices(name)
        .or_else(|| crate::ck3::tables::effects::effect_value_choices(name))
    { return r; }
    #[cfg(feature = "vic3")]
    if let r @ Some(_) = crate::vic3::tables::triggers::trigger_value_choices(name)
        .or_else(|| crate::vic3::tables::effects::effect_value_choices(name))
    { return r; }
    #[cfg(feature = "imperator")]
    if let r @ Some(_) = crate::imperator::tables::triggers::trigger_value_choices(name)
        .or_else(|| crate::imperator::tables::effects::effect_value_choices(name))
    { return r; }
    #[cfg(feature = "hoi4")]
    if let r @ Some(_) = crate::hoi4::tables::triggers::trigger_value_choices(name)
        .or_else(|| crate::hoi4::tables::effects::effect_value_choices(name))
    { return r; }
    #[cfg(feature = "eu5")]
    if let r @ Some(_) = crate::eu5::tables::triggers::trigger_value_choices(name)
        .or_else(|| crate::eu5::tables::effects::effect_value_choices(name))
    { return r; }
    None
}

// ─── Scope chain entries ──────────────────────────────────────────────────────

/// One entry in the scope navigation table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeChainEntry {
    /// The navigation keyword (e.g. `"capital_county"`, `"liege"`).
    pub name: &'static str,
    /// Human-readable source scope(s) (e.g. `"Character"`, `"Character|Province"`).
    pub from_scope: String,
    /// Human-readable destination scope (e.g. `"LandedTitle"`).
    pub to_scope: String,
}

/// All built-in scope navigation steps for the compiled game.
///
/// Used for scope-chain completion: after `.`, suggest entries where `from_scope` is
/// compatible with the current scope context.
#[allow(unreachable_code)]
pub fn scope_chain_entries() -> Vec<ScopeChainEntry> {
    fn convert(table: &[(impl std::fmt::Display, &'static str, impl std::fmt::Display)]) -> Vec<ScopeChainEntry> {
        table.iter().map(|(from, name, to)| ScopeChainEntry {
            name,
            from_scope: format!("{from}"),
            to_scope: format!("{to}"),
        }).collect()
    }
    #[cfg(feature = "ck3")]       { return convert(crate::ck3::tables::scope_transitions()); }
    #[cfg(feature = "vic3")]      { return convert(crate::vic3::tables::scope_transitions()); }
    #[cfg(feature = "imperator")] { return convert(crate::imperator::tables::scope_transitions()); }
    #[cfg(feature = "hoi4")]      { return convert(crate::hoi4::tables::scope_transitions()); }
    #[cfg(feature = "eu5")]       { return convert(crate::eu5::tables::scope_transitions()); }
    #[cfg(not(any(feature = "ck3", feature = "vic3", feature = "imperator", feature = "hoi4", feature = "eu5")))]
    vec![]
}

// ─── Entry types ──────────────────────────────────────────────────────────────

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

/// Return triggers and effects valid in the given scope, identified by its Debug name
/// (e.g. `"Character"`, `"Country"`, `"State"`).
///
/// Currently only HOI4 has explicit Iterator scopes. For other games this returns empty.
pub fn entries_for_scope(scope_hint: &str) -> Vec<LspEntry> {
    let mut out = Vec::new();
    #[cfg(feature = "hoi4")]
    {
        for name in crate::hoi4::tables::triggers::trigger_names_for_scope(scope_hint) {
            out.push(LspEntry { name: name.to_owned(), kind: LspEntryKind::Trigger });
        }
        for name in crate::hoi4::tables::effects::effect_names_for_scope(scope_hint) {
            out.push(LspEntry { name: name.to_owned(), kind: LspEntryKind::Effect });
        }
    }
    out
}
