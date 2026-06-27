//! Environment and sky configuration parsed from environment.def.
//!
//! The environment system controls time-of-day rendering including:
//! - Sky texture selection and blending
//! - Lighting lookup table rows
//! - Fog, clouds, water, and other atmospheric effects

use crate::def::binary::def_binary::{DefBinary, DefBody};
use crate::def::binary::names::Names;
use crate::def::{EnvironmentDef, EnvironmentThemeDef};
use crate::def::text::{
    Definition, DefParseError, Expr, PathSegment, Statement, parse_def_file,
};
use derive_more::{Display, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TimeKeyframe {
    pub time_of_day: f32,
    pub sky_texture_0: Option<String>,
    pub sky_texture_1: Option<String>,
    pub sky_texture_1_blend: f32,
    pub moon_lit: bool,
    pub fog_start_z: f32,
    pub fog_end_z: f32,
}

#[derive(Debug, Clone)]
pub struct EnvironmentTheme {
    pub name: String,
    pub keyframes: Vec<TimeKeyframe>,
}

impl EnvironmentTheme {
    pub fn sky_texture_names(&self) -> Vec<&str> {
        let mut names = Vec::new();
        for kf in &self.keyframes {
            if let Some(ref name) = kf.sky_texture_0
                && !names.contains(&name.as_str()) { names.push(name.as_str()); }
            if let Some(ref name) = kf.sky_texture_1
                && !names.contains(&name.as_str()) { names.push(name.as_str()); }
        }
        names
    }

    /// Find the keyframes bracketing `time` and the blend factor between them.
    ///
    /// Returns `None` when the theme has no keyframes (nothing to interpolate).
    pub fn keyframes_at_time(&self, time: f32) -> Option<(&TimeKeyframe, &TimeKeyframe, f32)> {
        if self.keyframes.is_empty() {
            return None;
        }
        let time = time.rem_euclid(24.0);
        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time_of_day <= time { prev_idx = i; }
        }
        let next_idx = (prev_idx + 1) % self.keyframes.len();
        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[next_idx];
        let prev_time = prev.time_of_day;
        let mut next_time = next.time_of_day;
        if next_time <= prev_time { next_time += 24.0; }
        let mut adjusted_time = time;
        if adjusted_time < prev_time { adjusted_time += 24.0; }
        let duration = next_time - prev_time;
        let blend = if duration > 0.0 { ((adjusted_time - prev_time) / duration).clamp(0.0, 1.0) } else { 0.0 };
        Some((prev, next, blend))
    }

    pub fn sky_textures_at_time(&self, time: f32) -> (Option<&str>, Option<&str>, f32) {
        let Some((prev, next, keyframe_blend)) = self.keyframes_at_time(time) else {
            return (None, None, 0.0);
        };
        if keyframe_blend < 0.5 {
            if prev.sky_texture_1.is_some() {
                (prev.sky_texture_0.as_deref(), prev.sky_texture_1.as_deref(), prev.sky_texture_1_blend)
            } else {
                (prev.sky_texture_0.as_deref(), next.sky_texture_0.as_deref(), keyframe_blend * 2.0)
            }
        } else if next.sky_texture_1.is_some() {
            (prev.sky_texture_0.as_deref(), next.sky_texture_0.as_deref(), (keyframe_blend - 0.5) * 2.0)
        } else {
            (prev.sky_texture_0.as_deref(), next.sky_texture_0.as_deref(), keyframe_blend)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnvironmentConfig {
    pub themes: HashMap<String, EnvironmentTheme>,
    pub environment_def: Option<EnvironmentDef>,
}

impl EnvironmentConfig {
    pub fn parse(input: &str) -> Result<Self, EnvironmentParseError> {
        let def_file = parse_def_file(input).map_err(EnvironmentParseError::Def)?;
        let mut themes = HashMap::new();
        for def in &def_file.definitions {
            if def.def_type == "ENVIRONMENT_THEME_DAY" {
                let theme = Self::parse_theme(&def.name, def)?;
                themes.insert(def.name.clone(), theme);
            }
        }
        Ok(Self { themes, environment_def: None })
    }

    pub fn from_def(env_def: EnvironmentDef) -> Self {
        Self {
            themes: HashMap::new(),
            environment_def: Some(env_def),
        }
    }

    pub fn from_binary_defs(
        def_binary: &DefBinary,
        names: &Names,
        resolve_sky_texture: impl Fn(i32) -> Option<String>,
    ) -> Self {
        let mut themes = HashMap::new();
        for entry in def_binary.entries(names) {
            if let DefBody::EnvironmentThemeDaySet(ref theme_set) = entry.record.body {
                let key = entry
                    .file_name
                    .or(entry.def_name)
                    .unwrap_or("unknown")
                    .to_string();
                let keyframes: Vec<TimeKeyframe> = theme_set
                    .time
                    .iter()
                    .map(|def| TimeKeyframe {
                        time_of_day: def.time_of_day,
                        moon_lit: def.moon_lit,
                        fog_start_z: def.fog_start_z,
                        fog_end_z: def.fog_end_z,
                        sky_texture_0: resolve_sky_texture(def.sky_texture_0),
                        sky_texture_1: resolve_sky_texture(def.sky_texture_1),
                        sky_texture_1_blend: def.sky_texture_1_blend,
                    })
                    .collect();
                themes.insert(
                    key.clone(),
                    EnvironmentTheme {
                        name: key,
                        keyframes,
                    },
                );
            }
        }
        Self {
            themes,
            environment_def: None,
        }
    }

    pub fn add_theme(&mut self, name: String, theme: &EnvironmentThemeDef) {
        let keyframe = TimeKeyframe {
            time_of_day: theme.time_of_day,
            moon_lit: theme.moon_lit,
            fog_start_z: theme.fog_start_z,
            fog_end_z: theme.fog_end_z,
            sky_texture_0: None,
            sky_texture_1: None,
            sky_texture_1_blend: theme.sky_texture_1_blend,
        };
        self.themes.insert(
            name.clone(),
            EnvironmentTheme {
                name,
                keyframes: vec![keyframe],
            },
        );
    }

    fn parse_theme(name: &str, def: &Definition) -> Result<EnvironmentTheme, EnvironmentParseError> {
        let mut keyframes_map: HashMap<i32, TimeKeyframe> = HashMap::new();
        for stmt in &def.body {
            let Statement::Field(field) = stmt else { continue };
            let segments = &field.path.segments;
            // We only care about `Time[idx].Property = ...` fields.
            if let [
                PathSegment::Field(field_name),
                PathSegment::Index(Expr::Integer(idx)),
                PathSegment::Field(prop),
                ..,
            ] = segments.as_slice()
                && field_name == "Time"
            {
                let keyframe = keyframes_map.entry(*idx as i32).or_default();
                Self::set_keyframe_property(keyframe, prop, &field.expr);
            }
        }
        let mut keyframes: Vec<_> = keyframes_map.into_iter().collect();
        keyframes.sort_by_key(|(idx, _)| *idx);
        let keyframes: Vec<_> = keyframes.into_iter().map(|(_, kf)| kf).collect();
        Ok(EnvironmentTheme { name: name.to_string(), keyframes })
    }

    fn set_keyframe_property(keyframe: &mut TimeKeyframe, prop: &str, expr: &Expr) {
        match prop {
            "TimeOfDay" => match expr {
                Expr::Float(f) => keyframe.time_of_day = *f,
                Expr::Integer(i) => keyframe.time_of_day = *i as f32,
                _ => {}
            },
            "SkyTexture0" => { if let Expr::Symbol(s) = expr { keyframe.sky_texture_0 = Some(s.clone()); } }
            "SkyTexture1" => { if let Expr::Symbol(s) = expr { keyframe.sky_texture_1 = Some(s.clone()); } }
            "SkyTexture1Blend" => match expr {
                Expr::Float(f) => keyframe.sky_texture_1_blend = *f,
                Expr::Integer(i) => keyframe.sky_texture_1_blend = *i as f32,
                _ => {}
            },
            "MoonLit" => { if let Expr::Bool(b) = expr { keyframe.moon_lit = *b; } }
            "FogStartZ" => match expr {
                Expr::Float(f) => keyframe.fog_start_z = *f,
                Expr::Integer(i) => keyframe.fog_start_z = *i as f32,
                _ => {}
            },
            "FogEndZ" => match expr {
                Expr::Float(f) => keyframe.fog_end_z = *f,
                Expr::Integer(i) => keyframe.fog_end_z = *i as f32,
                _ => {}
            },
            _ => {}
        }
    }
}

#[derive(Debug, Display, Error)]
pub enum EnvironmentParseError {
    #[display("def parse error: {_0}")]
    Def(DefParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_interpolation() {
        let theme = EnvironmentTheme {
            name: "test".to_string(),
            keyframes: vec![
                TimeKeyframe { time_of_day: 0.0, sky_texture_0: Some("MIDNIGHT".to_string()), ..Default::default() },
                TimeKeyframe { time_of_day: 6.0, sky_texture_0: Some("MORNING".to_string()), ..Default::default() },
                TimeKeyframe { time_of_day: 12.0, sky_texture_0: Some("MIDDAY".to_string()), ..Default::default() },
                TimeKeyframe { time_of_day: 18.0, sky_texture_0: Some("EVENING".to_string()), ..Default::default() },
            ],
        };
        let (prev, _next, blend) = theme.keyframes_at_time(0.0).unwrap();
        assert_eq!(prev.time_of_day, 0.0);
        assert_eq!(blend, 0.0);
        let (prev, next, blend) = theme.keyframes_at_time(3.0).unwrap();
        assert_eq!(prev.time_of_day, 0.0);
        assert_eq!(next.time_of_day, 6.0);
        assert!((blend - 0.5).abs() < 0.01);
        let (prev, _next, _blend) = theme.keyframes_at_time(12.0).unwrap();
        assert_eq!(prev.time_of_day, 12.0);
    }

    #[test]
    fn empty_theme_does_not_panic() {
        let theme = EnvironmentTheme { name: "empty".to_string(), keyframes: Vec::new() };
        assert!(theme.keyframes_at_time(6.0).is_none());
        assert_eq!(theme.sky_textures_at_time(6.0), (None, None, 0.0));
    }

    #[test]
    #[ignore]
    fn test_from_binary_defs() {
        use std::path::Path;

        let names_path = Path::new("/home/jamen/Fable/data/CompiledDefs/names.bin");
        let game_bin_path = Path::new("/home/jamen/Fable/data/CompiledDefs/game.bin");

        let names = Names::load(names_path).expect("Failed to load names.bin");
        let def_binary =
            DefBinary::load_with_names(game_bin_path, &names).expect("Failed to load game.bin");

        let config =
            EnvironmentConfig::from_binary_defs(&def_binary, &names, |id| {
                Some(format!("TEXTURE_{id}"))
            });

        let theme1 = config
            .themes
            .get("ENVIRONMENT_THEME1")
            .expect("ENVIRONMENT_THEME1 should exist");
        assert!(
            theme1.keyframes.len() > 0,
            "Theme should have at least one keyframe"
        );

        eprintln!("ENVIRONMENT_THEME1 keyframes ({}):", theme1.keyframes.len());
        for kf in &theme1.keyframes {
            eprintln!(
                "  time={:.1}, moon_lit={}, fog_start={:.1}, fog_end={:.1}, sky0={:?}, sky1={:?}, blend={:.2}",
                kf.time_of_day,
                kf.moon_lit,
                kf.fog_start_z,
                kf.fog_end_z,
                kf.sky_texture_0,
                kf.sky_texture_1,
                kf.sky_texture_1_blend,
            );
        }
    }
}
