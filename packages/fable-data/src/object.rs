use crate::def::text::{DefFile, parse_def_file};
use crate::def::{DefParseError, Definition, Expr, Spanned, Statement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectGraphicType {
    None,
    StaticMesh,
    AnimatedMesh,
    Sprite,
    GeneratedEffect,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectGraphic {
    pub mesh_symbol: Option<String>,
    pub graphic_type: ObjectGraphicType,
}

pub struct ObjectDefs {
    defs: DefFile,
}

impl ObjectDefs {
    pub fn parse(input: &str) -> Result<Self, DefParseError> {
        Ok(Self {
            defs: parse_def_file(input)?,
        })
    }

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
        self.defs
            .by_name
            .get(name)
            .map(|&idx| &self.defs.definitions[idx].value)
    }

    fn collect_properties(&self, def: &Definition) -> Vec<(String, String)> {
        let mut props = Vec::new();

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

        for ancestor in chain.iter().rev() {
            for stmt in &ancestor.body {
                if let Statement::Field(field) = &stmt.value {
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

fn expr_to_string(expr: &Spanned<Expr>) -> String {
    match &expr.value {
        Expr::String(s) => s.clone(),
        Expr::Symbol(s) => s.clone(),
        Expr::Number(s) => s.clone(),
        Expr::Bool(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        Expr::Constructor(c) => c.name.clone(),
        Expr::BitOr(_) | Expr::Add(_) => expr.value.to_string(),
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
