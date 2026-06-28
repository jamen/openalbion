//! Resolves `OBJECT` definitions to their mesh symbol and graphic type.
//!
//! Parses an `objects.def` file (the debug build's text format) and walks the `specialises`
//! inheritance chain to find each definition's `Graphic.BankIndex` (mesh symbol) and
//! `Graphic.Type` (static/animated/none).

use super::text::def_text::{Definition, Statement, parse_def_file};
use super::text::DefParseError;

/// The resolved graphic type for an object definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectGraphicType {
    None,
    StaticMesh,
    AnimatedMesh,
    Sprite,
    GeneratedEffect,
    Unknown,
}

/// Resolved mesh + graphic info for one object definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectGraphic {
    pub mesh_symbol: Option<String>,
    pub graphic_type: ObjectGraphicType,
}

/// A resolver that loads `objects.def` and provides mesh lookups by def name.
pub struct ObjectDefs {
    defs: super::text::def_text::DefFile,
}

impl ObjectDefs {
    pub fn parse(input: &str) -> Result<Self, DefParseError> {
        Ok(Self {
            defs: parse_def_file(input)?,
        })
    }

    /// Resolve an object definition by its instantiation name (e.g. `"OBJECT_MARKER"`),
    /// walking the `specialises` chain to find `Graphic.BankIndex` and `Graphic.Type`.
    pub fn resolve(&self, name: &str) -> Option<ObjectGraphic> {
        let def = self.find_def(name)?;
        let properties = self.collect_properties(def);

        let mesh_symbol = properties
            .iter()
            .find(|(k, _)| *k == "Graphic.BankIndex")
            .and_then(|(_, v)| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                }
            });

        let graphic_type = properties
            .iter()
            .find(|(k, _)| *k == "Graphic.Type")
            .map(|(_, v)| parse_graphic_type(v))
            .unwrap_or(ObjectGraphicType::None);

        Some(ObjectGraphic {
            mesh_symbol,
            graphic_type,
        })
    }

    fn find_def(&self, name: &str) -> Option<&Definition> {
        self.defs.by_name.get(name).map(|&idx| &self.defs.definitions[idx])
    }

    /// Walk `specialises` chain from `def` to collect all properties (later overrides earlier).
    fn collect_properties(&self, def: &Definition) -> Vec<(String, String)> {
        let mut props = Vec::new();

        // Walk the specialises chain — parent first, then child overrides.
        let mut chain: Vec<&Definition> = Vec::new();
        let mut current = def;
        loop {
            chain.push(current);
            match &current.specializes {
                Some(parent_name) => {
                    if let Some(parent) = self.find_def(parent_name) {
                        current = parent;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        // Apply in reverse order (root template first, leaf last so leaf overrides).
        for ancestor in chain.iter().rev() {
            for stmt in &ancestor.body {
                if let Statement::Field(field) = stmt {
                    props.push((field.path.to_string(), expr_to_string(&field.expr)));
                }
            }
        }

        props
    }
}

fn parse_graphic_type(s: &str) -> ObjectGraphicType {
    match s {
        "ENGINE_GRAPHIC_NULL" => ObjectGraphicType::None,
        "ENGINE_GRAPHIC_STATIC_MESH" => ObjectGraphicType::StaticMesh,
        "ENGINE_GRAPHIC_ANIMATING_MESH" => ObjectGraphicType::AnimatedMesh,
        "ENGINE_GRAPHIC_SPRITE" => ObjectGraphicType::Sprite,
        "ENGINE_GRAPHIC_GENERATED_EFFECT" => ObjectGraphicType::GeneratedEffect,
        _ => ObjectGraphicType::Unknown,
    }
}

fn expr_to_string(expr: &super::text::Expr) -> String {
    use super::text::Expr;
    match expr {
        Expr::String(s) => s.clone(),
        Expr::Symbol(s) => s.clone(),
        Expr::Integer(n) => n.to_string(),
        Expr::Float(x) => x.to_string(),
        Expr::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
        Expr::Constructor(c) => c.name.clone(),
        Expr::BitOr(_) | Expr::Add(_) => expr.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_simple_object() {
        let input = r#"#definition OBJECT OBJECT_TEST
    Graphic.Type ENGINE_GRAPHIC_STATIC_MESH;
    Graphic.BankIndex MESH_MARKER_01;
#end_definition
"#;
        let defs = ObjectDefs::parse(input).unwrap();
        let g = defs.resolve("OBJECT_TEST").unwrap();
        assert_eq!(g.mesh_symbol, Some("MESH_MARKER_01".into()));
        assert_eq!(g.graphic_type, ObjectGraphicType::StaticMesh);
    }

    #[test]
    fn resolve_with_inheritance() {
        let input = r#"#definition_template OBJECT OBJECT_TEMPLATE_TEST
    Graphic.Type ENGINE_GRAPHIC_STATIC_MESH;
    Material MATERIAL_WOOD;
#end_definition

#definition OBJECT OBJECT_CHILD_TEST specialises OBJECT_TEMPLATE_TEST
    Graphic.BankIndex MESH_BARREL_05;
#end_definition
"#;
        let defs = ObjectDefs::parse(input).unwrap();
        let g = defs.resolve("OBJECT_CHILD_TEST").unwrap();
        assert_eq!(g.mesh_symbol, Some("MESH_BARREL_05".into()));
        assert_eq!(g.graphic_type, ObjectGraphicType::StaticMesh);
    }
}
