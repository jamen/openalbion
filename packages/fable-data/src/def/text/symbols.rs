use super::header::{EnumDecl, EnumExpr, Header, HeaderItem};
use std::collections::HashMap;

pub struct SymbolTable {
    map: HashMap<String, i64>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
    pub fn lookup(&self, name: &str) -> Option<i64> {
        self.map.get(name).copied()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, i64)> {
        self.map.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

#[derive(Debug)]
pub enum SymbolEvalError {
    UnknownSymbol(String),
    DuplicateSymbol(String),
    InvalidShift(i64),
}

impl SymbolTable {
    pub fn evaluate(&mut self, header: &Header) -> Result<(), SymbolEvalError> {
        for item in &header.items {
            self.evaluate_header_item(item)?;
        }
        Ok(())
    }
    fn evaluate_header_item(&mut self, item: &HeaderItem) -> Result<(), SymbolEvalError> {
        use HeaderItem as I;
        match item {
            I::Enum(decl) => self.evaluate_enum(decl),
            I::Define(d) => self.insert(&d.name, d.value),
            I::Namespace(ns) => {
                for item in &ns.items { self.evaluate_header_item(item)?; }
                Ok(())
            }
            I::IfDef(ifdef) => {
                let branch = if self.is_defined(&ifdef.condition) { &ifdef.if_branch }
                else { ifdef.else_branch.as_deref().unwrap_or(&[]) };
                for item in branch { self.evaluate_header_item(item)?; }
                Ok(())
            }
        }
    }
    fn evaluate_enum(&mut self, decl: &EnumDecl) -> Result<(), SymbolEvalError> {
        let mut last_value: Option<i64> = None;
        for variant in &decl.variants {
            let value = match &variant.value {
                Some(expr) => self.evaluate_enum_expr(expr)?,
                None => last_value.map_or(0, |v| v + 1),
            };
            self.insert(&variant.name, value)?;
            last_value = Some(value);
        }
        Ok(())
    }
    pub fn insert(&mut self, name: &str, value: i64) -> Result<(), SymbolEvalError> {
        if self.map.contains_key(name) { return Err(SymbolEvalError::DuplicateSymbol(name.to_string())); }
        self.map.insert(name.to_string(), value);
        Ok(())
    }
    fn is_defined(&self, cond: &str) -> bool { cond == "_WINDOWS" }
    fn evaluate_enum_expr(&self, expr: &EnumExpr) -> Result<i64, SymbolEvalError> {
        use EnumExpr as E;
        match expr {
            E::Int(n) => Ok(*n),
            E::Ident(name) => self.lookup(name).ok_or_else(|| SymbolEvalError::UnknownSymbol(name.clone())),
            E::Shift(terms) => {
                let mut iter = terms.iter();
                let first = self.evaluate_enum_expr(iter.next().unwrap())?;
                iter.try_fold(first, |acc, term| {
                    let n = self.evaluate_enum_expr(term)?;
                    if !(0..64).contains(&n) { return Err(SymbolEvalError::InvalidShift(n)); }
                    Ok(acc << n)
                })
            }
            E::BitOr(terms) => terms.iter().map(|t| self.evaluate_enum_expr(t)).try_fold(0i64, |acc, v| Ok(acc | v?)),
        }
    }
}
