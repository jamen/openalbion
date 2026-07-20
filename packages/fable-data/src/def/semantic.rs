//! Reference-resolving semantic value tree for def bodies.
//!
//! A [`SemVal`] is a fully-decoded, owned tree of a def body, produced by
//! walking its typed fields through the reflection layer
//! ([`VisitFields`]/[`FieldRef`]) and resolving `DefIndex`/`DefString`
//! references to names/strings via a caller-supplied [`Resolvers`]. Comparing
//! two `SemVal`s answers *"do these encode the same def data?"* independently of
//! the concrete global-index / name-offset assignment.
//!
//! This is the foundation of from-scratch verification: our from-scratch
//! `game.bin` assigns its own indices, so a raw byte-compare against retail
//! fails on every reference even when the data is identical. Resolving
//! references to names before comparing removes that noise. In "Mode A"
//! (reference-ordered) the identity resolvers ([`Resolvers::raw`]) render
//! references as raw `idx:<n>`, so `SemVal` equality reduces to value equality
//! over the same index space.
//!
//! [`DiffPolicy`] optionally compares maps / lists as unordered multisets,
//! which is how the ledger distinguishes a genuine data difference (`BUG`) from
//! a mere reordering artifact (MSVC `std::sort` tie-breaks on duplicate keys →
//! `ACCEPT_SORT`).

use crate::def::dispatch::GameBody;
use crate::def::visit::{FieldRef, FieldVisitor, StructSlot, VisitFields};

/// A decoded, owned, comparable def-field value.
#[derive(Debug, Clone, PartialEq)]
pub enum SemVal {
    /// `f32` as raw bits — retail floats reproduce bit-for-bit, and bits are
    /// `Eq`/`Hash`able and NaN-safe for comparison.
    F32(u32),
    /// any integer / enum / flags value widened to `i64`.
    Int(i64),
    Bool(bool),
    Str(String),
    WStr(String),
    /// resolved def-index reference (name if resolvable, else `idx:<n>`).
    DefRef(String),
    /// resolved def-string (value if resolvable, else `off:<n>`).
    DefStr(String),
    PString(Vec<u8>),
    List(Vec<SemVal>),
    Map(Vec<(SemVal, SemVal)>),
    Struct(Vec<(&'static str, SemVal)>),
    Variant(u32, Vec<(&'static str, SemVal)>),
    /// A field the reflection can't read (fixed arrays, unmodeled `Complex`).
    /// Content is opaque here — two `Opaque`s compare equal by tag, so a caller
    /// must not treat `SemVal` equality as *proven* data-equality when the tree
    /// [`contains_opaque`](SemVal::contains_opaque). Byte-compare covers these.
    Opaque(&'static str),
}

impl SemVal {
    /// Whether any node is [`SemVal::Opaque`] (an unreadable region whose
    /// equality can't be decided structurally).
    pub fn contains_opaque(&self) -> bool {
        match self {
            SemVal::Opaque(_) => true,
            SemVal::List(xs) => xs.iter().any(SemVal::contains_opaque),
            SemVal::Map(xs) => xs
                .iter()
                .any(|(k, v)| k.contains_opaque() || v.contains_opaque()),
            SemVal::Struct(xs) | SemVal::Variant(_, xs) => {
                xs.iter().any(|(_, v)| v.contains_opaque())
            }
            _ => false,
        }
    }
}

/// Resolves references to their stable identity (name / string value) so two
/// bodies over different index spaces can be compared.
pub struct Resolvers<'a> {
    /// def global index → def name.
    pub def_index_name: &'a dyn Fn(i32) -> Option<String>,
    /// def-string name-table offset → string value.
    pub def_string_value: &'a dyn Fn(i32) -> Option<String>,
}

fn resolve_none(_: i32) -> Option<String> {
    None
}

impl Resolvers<'static> {
    /// Identity resolvers: references render as raw `idx:<n>` / `off:<n>`, so
    /// two bodies over the SAME index space (Mode A) compare exactly by value.
    pub fn raw() -> Self {
        Resolvers { def_index_name: &resolve_none, def_string_value: &resolve_none }
    }
}

/// Decode one field into a [`SemVal`].
pub fn field_to_semval(field: FieldRef<'_>, r: &Resolvers) -> SemVal {
    match field {
        FieldRef::F32(x) => SemVal::F32(x.to_bits()),
        FieldRef::I32(x) => SemVal::Int(*x as i64),
        FieldRef::U32(x) => SemVal::Int(*x as i64),
        FieldRef::Bool(x) => SemVal::Bool(*x),
        FieldRef::Str(x) => SemVal::Str(x.clone()),
        FieldRef::WStr(x) => SemVal::WStr(x.0.clone()),
        FieldRef::Enum(s) => SemVal::Int(s.get_i32() as i64),
        FieldRef::Flags(s) => SemVal::Int(s.get_i32() as i64),
        FieldRef::DefString(ds) => {
            let off = ds.0;
            SemVal::DefStr((r.def_string_value)(off).unwrap_or_else(|| format!("off:{off}")))
        }
        FieldRef::DefIndex(di) => {
            let idx = di.0;
            SemVal::DefRef((r.def_index_name)(idx).unwrap_or_else(|| format!("idx:{idx}")))
        }
        FieldRef::U8(x) => SemVal::Int(*x as i64),
        FieldRef::U16(x) => SemVal::Int(*x as i64),
        FieldRef::U64(x) => SemVal::Int(*x as i64),
        FieldRef::I8(x) => SemVal::Int(*x as i64),
        FieldRef::I16(x) => SemVal::Int(*x as i64),
        FieldRef::PString(p) => SemVal::PString(p.0.clone()),
        FieldRef::Vec(slot) => {
            let n = slot.len();
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                out.push(field_to_semval(slot.element(i), r));
            }
            SemVal::List(out)
        }
        FieldRef::Map(slot) => {
            let mut out = Vec::new();
            slot.for_each_pair(&mut |k, v| {
                out.push((field_to_semval(k, r), field_to_semval(v, r)));
            });
            SemVal::Map(out)
        }
        FieldRef::Struct(slot) => SemVal::Struct(read_members(slot, r)),
        FieldRef::Variant(slot) => {
            let tag = slot.tag();
            let n = slot.member_count();
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                let name = slot.member_name(i).unwrap_or("?");
                if let Some(m) = slot.member(i) {
                    out.push((name, field_to_semval(m, r)));
                }
            }
            SemVal::Variant(tag, out)
        }
        FieldRef::Complex(s) => SemVal::Opaque(s),
    }
}

fn read_members(slot: &mut dyn StructSlot, r: &Resolvers) -> Vec<(&'static str, SemVal)> {
    let n = slot.member_count();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let name = slot.member_name(i).unwrap_or("?");
        if let Some(m) = slot.member(i) {
            out.push((name, field_to_semval(m, r)));
        }
    }
    out
}

struct Collector<'a, 'b> {
    r: &'a Resolvers<'b>,
    out: Vec<(&'static str, SemVal)>,
}

impl FieldVisitor for Collector<'_, '_> {
    fn field(&mut self, name: &'static str, field: FieldRef<'_>) {
        self.out.push((name, field_to_semval(field, self.r)));
    }
}

/// Decode any [`VisitFields`] def struct into a [`SemVal::Struct`].
pub fn to_semval<T: VisitFields>(v: &mut T, r: &Resolvers) -> SemVal {
    let mut c = Collector { r, out: Vec::new() };
    v.visit_fields(&mut c);
    SemVal::Struct(c.out)
}

/// Decode a [`GameBody`]'s active variant into a [`SemVal::Struct`].
pub fn game_body_to_semval(b: &mut GameBody, r: &Resolvers) -> SemVal {
    let mut c = Collector { r, out: Vec::new() };
    b.visit_active(&mut c);
    SemVal::Struct(c.out)
}

/// How to compare containers.
#[derive(Clone, Copy, Default)]
pub struct DiffPolicy {
    /// Compare `Map` pairs as an unordered multiset. C++ `std::map`/`std::set`
    /// are key-sorted, but MSVC `std::sort` tie-breaks on duplicate keys are
    /// order-unstable and unreproducible; treating maps as multisets classifies
    /// those as reordering (not a data difference).
    pub unordered_maps: bool,
    /// Compare `List` elements as an unordered multiset (same rationale for
    /// duplicate-key vectors like `CVectorMap`).
    pub unordered_lists: bool,
}

impl DiffPolicy {
    /// Exact structural comparison (order matters everywhere).
    pub fn strict() -> Self {
        DiffPolicy { unordered_maps: false, unordered_lists: false }
    }
    /// Order-insensitive for maps and lists.
    pub fn unordered() -> Self {
        DiffPolicy { unordered_maps: true, unordered_lists: true }
    }
}

/// Structural equality under `policy`.
pub fn sem_eq(a: &SemVal, b: &SemVal, policy: DiffPolicy) -> bool {
    match (a, b) {
        (SemVal::Map(xa), SemVal::Map(xb)) => {
            xa.len() == xb.len()
                && if policy.unordered_maps {
                    multiset_eq_pairs(xa, xb, policy)
                } else {
                    xa.iter()
                        .zip(xb)
                        .all(|((ka, va), (kb, vb))| sem_eq(ka, kb, policy) && sem_eq(va, vb, policy))
                }
        }
        (SemVal::List(xa), SemVal::List(xb)) => {
            xa.len() == xb.len()
                && if policy.unordered_lists {
                    multiset_eq(xa, xb, policy)
                } else {
                    xa.iter().zip(xb).all(|(x, y)| sem_eq(x, y, policy))
                }
        }
        (SemVal::Struct(xa), SemVal::Struct(xb)) => {
            xa.len() == xb.len()
                && xa
                    .iter()
                    .zip(xb)
                    .all(|((na, va), (nb, vb))| na == nb && sem_eq(va, vb, policy))
        }
        (SemVal::Variant(ta, xa), SemVal::Variant(tb, xb)) => {
            ta == tb
                && xa.len() == xb.len()
                && xa
                    .iter()
                    .zip(xb)
                    .all(|((na, va), (nb, vb))| na == nb && sem_eq(va, vb, policy))
        }
        _ => a == b,
    }
}

fn multiset_eq(a: &[SemVal], b: &[SemVal], policy: DiffPolicy) -> bool {
    let mut used = vec![false; b.len()];
    for x in a {
        let Some(j) = b.iter().enumerate().position(|(j, y)| !used[j] && sem_eq(x, y, policy)) else {
            return false;
        };
        used[j] = true;
    }
    true
}

fn multiset_eq_pairs(a: &[(SemVal, SemVal)], b: &[(SemVal, SemVal)], policy: DiffPolicy) -> bool {
    let mut used = vec![false; b.len()];
    for (ka, va) in a {
        let Some(j) = b.iter().enumerate().position(|(j, (kb, vb))| {
            !used[j] && sem_eq(ka, kb, policy) && sem_eq(va, vb, policy)
        }) else {
            return false;
        };
        used[j] = true;
    }
    true
}

/// Dotted path to the first structural divergence under `policy`, with a short
/// rendering of each side — for `BUG` diagnostics. `None` when equal.
pub fn first_diff(a: &SemVal, b: &SemVal, policy: DiffPolicy) -> Option<Diff> {
    let mut path = String::new();
    first_diff_at(a, b, policy, &mut path)
}

/// A located structural difference.
#[derive(Debug, Clone)]
pub struct Diff {
    pub path: String,
    pub ours: String,
    pub theirs: String,
}

fn short(v: &SemVal) -> String {
    match v {
        SemVal::List(xs) => format!("List(len={})", xs.len()),
        SemVal::Map(xs) => format!("Map(len={})", xs.len()),
        SemVal::Struct(xs) => format!("Struct(fields={})", xs.len()),
        SemVal::Variant(t, xs) => format!("Variant(tag={t},fields={})", xs.len()),
        other => format!("{other:?}"),
    }
}

fn diff_here(a: &SemVal, b: &SemVal, path: &str) -> Diff {
    Diff { path: path.to_string(), ours: short(a), theirs: short(b) }
}

fn diff_members(
    a: &SemVal,
    b: &SemVal,
    xa: &[(&'static str, SemVal)],
    xb: &[(&'static str, SemVal)],
    policy: DiffPolicy,
    path: &mut String,
) -> Option<Diff> {
    if xa.len() != xb.len() {
        return Some(diff_here(a, b, path));
    }
    for ((na, va), (nb, vb)) in xa.iter().zip(xb) {
        if na != nb {
            return Some(diff_here(a, b, path));
        }
        let len = path.len();
        if !path.is_empty() {
            path.push('.');
        }
        path.push_str(na);
        if let Some(d) = first_diff_at(va, vb, policy, path) {
            return Some(d);
        }
        path.truncate(len);
    }
    None
}

fn first_diff_at(a: &SemVal, b: &SemVal, policy: DiffPolicy, path: &mut String) -> Option<Diff> {
    match (a, b) {
        (SemVal::Struct(xa), SemVal::Struct(xb)) => diff_members(a, b, xa, xb, policy, path),
        (SemVal::Variant(ta, xa), SemVal::Variant(tb, xb)) => {
            if ta != tb {
                return Some(diff_here(a, b, path));
            }
            diff_members(a, b, xa, xb, policy, path)
        }
        // For diagnostics, descend positionally into equal-length lists/maps
        // even under an unordered policy: `first_diff` is only ever called on
        // trees already known unequal, and positional descent yields a precise
        // leaf path (the equality *verdict* still comes from `sem_eq`).
        (SemVal::List(xa), SemVal::List(xb)) if xa.len() == xb.len() => {
            for (i, (x, y)) in xa.iter().zip(xb).enumerate() {
                let len = path.len();
                path.push_str(&format!("[{i}]"));
                if let Some(d) = first_diff_at(x, y, policy, path) {
                    return Some(d);
                }
                path.truncate(len);
            }
            // equal element-wise but caller says unequal ⇒ pure reordering.
            None
        }
        (SemVal::Map(xa), SemVal::Map(xb)) if xa.len() == xb.len() => {
            for (i, ((ka, va), (kb, vb))) in xa.iter().zip(xb).enumerate() {
                let len = path.len();
                path.push_str(&format!("[{i}].key"));
                if let Some(d) = first_diff_at(ka, kb, policy, path) {
                    return Some(d);
                }
                path.truncate(len);
                path.push_str(&format!("[{i}]"));
                if let Some(d) = first_diff_at(va, vb, policy, path) {
                    return Some(d);
                }
                path.truncate(len);
            }
            None
        }
        _ if sem_eq(a, b, policy) => None,
        _ => Some(diff_here(a, b, path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[(&'static str, SemVal)]) -> SemVal {
        SemVal::Struct(v.to_vec())
    }

    #[test]
    fn scalars_and_refs() {
        assert!(sem_eq(&SemVal::Int(3), &SemVal::Int(3), DiffPolicy::strict()));
        assert!(!sem_eq(&SemVal::Int(3), &SemVal::Int(4), DiffPolicy::strict()));
        assert!(sem_eq(
            &SemVal::DefRef("OBJECT_X".into()),
            &SemVal::DefRef("OBJECT_X".into()),
            DiffPolicy::strict()
        ));
    }

    #[test]
    fn maps_unordered_vs_strict() {
        let a = SemVal::Map(vec![
            (SemVal::Int(1), SemVal::Int(10)),
            (SemVal::Int(2), SemVal::Int(20)),
        ]);
        let b = SemVal::Map(vec![
            (SemVal::Int(2), SemVal::Int(20)),
            (SemVal::Int(1), SemVal::Int(10)),
        ]);
        assert!(!sem_eq(&a, &b, DiffPolicy::strict()));
        assert!(sem_eq(&a, &b, DiffPolicy::unordered()));
    }

    #[test]
    fn opaque_detection() {
        let v = s(&[("blob", SemVal::Opaque("array"))]);
        assert!(v.contains_opaque());
        // two Opaques compare equal by tag (byte-compare must cover them).
        assert!(sem_eq(
            &SemVal::Opaque("array"),
            &SemVal::Opaque("array"),
            DiffPolicy::strict()
        ));
    }

    #[test]
    fn first_diff_path() {
        let a = s(&[("x", SemVal::Int(1)), ("y", SemVal::Int(2))]);
        let b = s(&[("x", SemVal::Int(1)), ("y", SemVal::Int(9))]);
        let d = first_diff(&a, &b, DiffPolicy::strict()).unwrap();
        assert_eq!(d.path, "y");
    }
}
