//! Environment and sky configuration parsed from environment.def.
//!
//! The environment system controls time-of-day rendering including:
//! - Sky texture selection and blending
//! - Lighting lookup table rows
//! - Fog, clouds, water, and other atmospheric effects

use crate::def::{Definition, ParseError, PathSegment, Statement, Value, parse_def_file};
use derive_more::{Display, Error};
use std::collections::HashMap;

/// A time-of-day keyframe defining sky appearance at a specific hour.
#[derive(Debug, Clone, Default)]
pub struct TimeKeyframe {
    /// Hour of day (0-24).
    pub time_of_day: f32,
    /// Primary sky texture name.
    pub sky_texture_0: Option<String>,
    /// Secondary sky texture name (for blending).
    pub sky_texture_1: Option<String>,
    /// Blend factor for secondary texture (0.0 = all texture0, 1.0 = all texture1).
    pub sky_texture_1_blend: f32,
    /// Whether the moon provides lighting.
    pub moon_lit: bool,
    /// Fog start distance.
    pub fog_start_z: f32,
    /// Fog end distance.
    pub fog_end_z: f32,
}

/// An environment theme defining a full day/night cycle.
#[derive(Debug, Clone)]
pub struct EnvironmentTheme {
    /// Theme name (e.g., "ENVIRONMENT_THEME1").
    pub name: String,
    /// Time keyframes (usually 8).
    pub keyframes: Vec<TimeKeyframe>,
}

impl EnvironmentTheme {
    /// Get all unique sky texture names used by this theme.
    pub fn sky_texture_names(&self) -> Vec<&str> {
        let mut names = Vec::new();
        for kf in &self.keyframes {
            if let Some(ref name) = kf.sky_texture_0 {
                if !names.contains(&name.as_str()) {
                    names.push(name.as_str());
                }
            }
            if let Some(ref name) = kf.sky_texture_1 {
                if !names.contains(&name.as_str()) {
                    names.push(name.as_str());
                }
            }
        }
        names
    }

    /// Find the two keyframes surrounding a given time and compute blend factor.
    ///
    /// Returns (prev_keyframe, next_keyframe, blend_factor) where blend_factor
    /// is 0.0 at prev_keyframe's time and 1.0 at next_keyframe's time.
    pub fn keyframes_at_time(&self, time: f32) -> (&TimeKeyframe, &TimeKeyframe, f32) {
        let time = time.rem_euclid(24.0);

        // Find the keyframe just before or at this time
        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time_of_day <= time {
                prev_idx = i;
            }
        }

        let next_idx = (prev_idx + 1) % self.keyframes.len();
        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[next_idx];

        // Calculate blend factor
        let prev_time = prev.time_of_day;
        let mut next_time = next.time_of_day;

        // Handle wrap-around (e.g., 21:00 to 00:00)
        if next_time <= prev_time {
            next_time += 24.0;
        }

        let mut adjusted_time = time;
        if adjusted_time < prev_time {
            adjusted_time += 24.0;
        }

        let duration = next_time - prev_time;
        let blend = if duration > 0.0 {
            ((adjusted_time - prev_time) / duration).clamp(0.0, 1.0)
        } else {
            0.0
        };

        (prev, next, blend)
    }

    /// Get the sky textures and blend factors for a given time.
    ///
    /// Returns (texture0_name, texture1_name, blend_factor) where the blend
    /// is computed from the keyframe interpolation and any intra-keyframe
    /// blending defined in the keyframe itself.
    pub fn sky_textures_at_time(&self, time: f32) -> (Option<&str>, Option<&str>, f32) {
        let (prev, next, keyframe_blend) = self.keyframes_at_time(time);

        // If we're close to a keyframe with its own blend, use that
        if keyframe_blend < 0.5 {
            // Closer to prev keyframe
            if prev.sky_texture_1.is_some() {
                // Keyframe has its own blend
                (
                    prev.sky_texture_0.as_deref(),
                    prev.sky_texture_1.as_deref(),
                    prev.sky_texture_1_blend,
                )
            } else {
                // Blend between prev and next keyframe's primary textures
                (
                    prev.sky_texture_0.as_deref(),
                    next.sky_texture_0.as_deref(),
                    keyframe_blend * 2.0, // Scale since we're in first half
                )
            }
        } else {
            // Closer to next keyframe
            if next.sky_texture_1.is_some() {
                // Next keyframe has its own blend, transition toward it
                (
                    prev.sky_texture_0.as_deref(),
                    next.sky_texture_0.as_deref(),
                    (keyframe_blend - 0.5) * 2.0,
                )
            } else {
                // Blend between prev and next keyframe's primary textures
                (
                    prev.sky_texture_0.as_deref(),
                    next.sky_texture_0.as_deref(),
                    keyframe_blend,
                )
            }
        }
    }
}

/// Parsed environment configuration from environment.def.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentConfig {
    /// All environment themes, keyed by name.
    pub themes: HashMap<String, EnvironmentTheme>,
}

impl EnvironmentConfig {
    /// Parse environment configuration from a def file string.
    pub fn parse(input: &str) -> Result<Self, EnvironmentParseError> {
        let def_file = parse_def_file(input).map_err(EnvironmentParseError::Def)?;

        let mut themes = HashMap::new();

        for (name, def) in def_file.definitions {
            if def.def_type == "ENVIRONMENT_THEME_DAY" {
                let theme = Self::parse_theme(&name, &def)?;
                themes.insert(name, theme);
            }
        }

        Ok(Self { themes })
    }

    fn parse_theme(
        name: &str,
        def: &Definition,
    ) -> Result<EnvironmentTheme, EnvironmentParseError> {
        let mut keyframes_map: HashMap<i32, TimeKeyframe> = HashMap::new();

        for stmt in &def.body {
            if let Statement::Assignment(assignment) = stmt {
                // Look for Time[N].Property patterns
                let segments = &assignment.path.segments;
                if segments.len() >= 2 {
                    if let (PathSegment::Field(field), PathSegment::Index(idx)) =
                        (&segments[0], &segments[1])
                    {
                        if field == "Time" {
                            let keyframe = keyframes_map.entry(*idx).or_default();

                            if segments.len() >= 3 {
                                if let PathSegment::Field(prop) = &segments[2] {
                                    Self::set_keyframe_property(keyframe, prop, &assignment.value);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert map to sorted vec
        let mut keyframes: Vec<_> = keyframes_map.into_iter().collect();
        keyframes.sort_by_key(|(idx, _)| *idx);
        let keyframes: Vec<_> = keyframes.into_iter().map(|(_, kf)| kf).collect();

        Ok(EnvironmentTheme {
            name: name.to_string(),
            keyframes,
        })
    }

    fn set_keyframe_property(keyframe: &mut TimeKeyframe, prop: &str, value: &Value) {
        match prop {
            "TimeOfDay" => {
                if let Some(v) = value.as_float() {
                    keyframe.time_of_day = v as f32;
                } else if let Some(v) = value.as_integer() {
                    keyframe.time_of_day = v as f32;
                }
            }
            "SkyTexture0" => {
                if let Some(v) = value.as_identifier() {
                    keyframe.sky_texture_0 = Some(v.to_string());
                }
            }
            "SkyTexture1" => {
                if let Some(v) = value.as_identifier() {
                    keyframe.sky_texture_1 = Some(v.to_string());
                }
            }
            "SkyTexture1Blend" => {
                if let Some(v) = value.as_float() {
                    keyframe.sky_texture_1_blend = v as f32;
                }
            }
            "MoonLit" => {
                if let Some(v) = value.as_bool() {
                    keyframe.moon_lit = v;
                }
            }
            "FogStartZ" => {
                if let Some(v) = value.as_float() {
                    keyframe.fog_start_z = v as f32;
                } else if let Some(v) = value.as_integer() {
                    keyframe.fog_start_z = v as f32;
                }
            }
            "FogEndZ" => {
                if let Some(v) = value.as_float() {
                    keyframe.fog_end_z = v as f32;
                } else if let Some(v) = value.as_integer() {
                    keyframe.fog_end_z = v as f32;
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Display, Error)]
pub enum EnvironmentParseError {
    #[display("def parse error: {_0}")]
    Def(ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_interpolation() {
        let theme = EnvironmentTheme {
            name: "test".to_string(),
            keyframes: vec![
                TimeKeyframe {
                    time_of_day: 0.0,
                    sky_texture_0: Some("MIDNIGHT".to_string()),
                    ..Default::default()
                },
                TimeKeyframe {
                    time_of_day: 6.0,
                    sky_texture_0: Some("MORNING".to_string()),
                    ..Default::default()
                },
                TimeKeyframe {
                    time_of_day: 12.0,
                    sky_texture_0: Some("MIDDAY".to_string()),
                    ..Default::default()
                },
                TimeKeyframe {
                    time_of_day: 18.0,
                    sky_texture_0: Some("EVENING".to_string()),
                    ..Default::default()
                },
            ],
        };

        // At midnight
        let (prev, next, blend) = theme.keyframes_at_time(0.0);
        assert_eq!(prev.time_of_day, 0.0);
        assert_eq!(blend, 0.0);

        // At 3am (halfway between midnight and morning)
        let (prev, next, blend) = theme.keyframes_at_time(3.0);
        assert_eq!(prev.time_of_day, 0.0);
        assert_eq!(next.time_of_day, 6.0);
        assert!((blend - 0.5).abs() < 0.01);

        // At noon
        let (prev, next, blend) = theme.keyframes_at_time(12.0);
        assert_eq!(prev.time_of_day, 12.0);
    }
}
