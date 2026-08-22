//! What you can type, and what it parses to.
//!
//! **Fable's own vocabulary, in spirit rather than in fact** (AGENTS.md §13.5, §3.9). The
//! original engine gives every `CEngineComponent` a `GetConsoleEnableFunctionName`, so
//! `ego_r.exe` answers to `EnableLandscape`, `EnableStaticMeshes` and eighteen more. We are not
//! reproducing that console — retail's is stripped and chasing it is explicitly ruled out — but
//! the *names* are worth keeping, because they are the ones already written down everywhere this
//! project's notes discuss subsystem isolation.
//!
//! Names are matched case-insensitively: `EnableSky`, `enablesky` and `ENABLESKY` are one
//! command. Nothing here touches the renderer — parsing a command and performing it are separate
//! so this half needs no GPU to test (§6.9).

use derive_more::Display;

/// A subsystem the console can switch off. One per world pass (`renderer::RenderToggles`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Subsystem {
    Sky,
    Landscape,
    StaticMeshes,
    RepeatedMeshes,
}

impl Subsystem {
    pub const ALL: [Subsystem; 4] = [
        Subsystem::Sky,
        Subsystem::Landscape,
        Subsystem::StaticMeshes,
        Subsystem::RepeatedMeshes,
    ];

    /// The command name, which is `Enable` plus this — matching the original's
    /// `Enable{Sky,Landscape,StaticMeshes,RepeatedMeshes}` (§3.9).
    pub fn name(self) -> &'static str {
        match self {
            Subsystem::Sky => "Sky",
            Subsystem::Landscape => "Landscape",
            Subsystem::StaticMeshes => "StaticMeshes",
            Subsystem::RepeatedMeshes => "RepeatedMeshes",
        }
    }

    /// What the pass draws, for `Help`. Written out rather than derived from the name, which
    /// would print "repeatedmeshes" at people.
    pub fn describes(self) -> &'static str {
        match self {
            Subsystem::Sky => "the sky dome and base band",
            Subsystem::Landscape => "the terrain's foreground layers",
            Subsystem::StaticMeshes => "the level's .tng things",
            Subsystem::RepeatedMeshes => "local detail's foliage",
        }
    }
}

/// One parsed command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// List every command, with its usage.
    Help,
    /// Empty the scrollback.
    Clear,
    /// Print what the renderer currently holds — the numbers `--text-demo` used to show.
    Stats,
    /// Turn the always-on stats overlay on, off, or over.
    ShowStats(Option<bool>),
    /// `Enable<Subsystem> [true|false]`. `None` means "flip whatever it is now".
    Enable(Subsystem, Option<bool>),
}

/// Why a line did not become a [`Command`]. Displayed straight into the scrollback, so the
/// wording is the error message.
#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum ParseError {
    #[display("unknown command `{_0}` — type Help for the list")]
    Unknown(String),
    #[display("`{value}` is not true or false")]
    NotABool { value: String },
    #[display("{name} takes at most one argument")]
    TooManyArguments { name: &'static str },
}

/// One row of the command table: what to type, and what it does.
///
/// A table rather than a `match` over names alone so that `Help` cannot drift out of date —
/// adding a command means adding a row here, and the help text is generated from the same rows
/// the parser reads.
pub struct CommandSpec {
    pub usage: &'static str,
    pub help: &'static str,
}

/// Every command except the `Enable*` family, which [`help_lines`] generates from
/// [`Subsystem::ALL`] so a new subsystem cannot be added to one and forgotten in the other.
const COMMANDS: [CommandSpec; 4] = [
    CommandSpec {
        usage: "Help",
        help: "list these commands",
    },
    CommandSpec {
        usage: "Clear",
        help: "empty the scrollback",
    },
    CommandSpec {
        usage: "Stats",
        help: "print what the renderer is holding",
    },
    CommandSpec {
        usage: "ShowStats [true|false]",
        help: "keep those numbers on screen",
    },
];

/// The `Help` output, one line per command.
///
/// The usage column is wide enough for the longest of them —
/// `EnableRepeatedMeshes [true|false]`, 33 characters — because a `{:<n}` narrower than its
/// content silently stops aligning rather than wrapping.
const USAGE_WIDTH: usize = 34;

pub fn help_lines() -> Vec<String> {
    let mut lines: Vec<String> = COMMANDS
        .iter()
        .map(|spec| format!("  {:<USAGE_WIDTH$}{}", spec.usage, spec.help))
        .collect();
    for subsystem in Subsystem::ALL {
        lines.push(format!(
            "  {:<USAGE_WIDTH$}{}",
            format!("Enable{} [true|false]", subsystem.name()),
            format!("draw {}", subsystem.describes()),
        ));
    }
    lines.push(String::new());
    // Flush left, unlike the rows above: everything indented by two spaces is something you can
    // type, and `every_line_help_prints_is_a_command_that_parses` relies on that.
    lines.push("With no argument, a toggle flips.".to_string());
    lines
}

/// `true`/`false`, and the spellings a dev console should not make you think about.
///
/// The original's console takes `TRUE`/`FALSE` (§3.9); `on`/`off`, `1`/`0` and `yes`/`no` are
/// ours, because typing `EnableSky off` and being told it is not a bool would be a small
/// daily annoyance for no gain.
fn parse_bool(text: &str) -> Option<bool> {
    match text.to_ascii_lowercase().as_str() {
        "true" | "on" | "1" | "yes" => Some(true),
        "false" | "off" | "0" | "no" => Some(false),
        _ => None,
    }
}

/// Parse one line. Whitespace-separated; there are no quotes and no escapes, because no command
/// takes a string (§13.5 — rudimentary on purpose).
///
/// An all-whitespace line is `Ok(None)`: pressing Enter on an empty prompt is not an error.
pub fn parse(line: &str) -> Result<Option<Command>, ParseError> {
    let mut words = line.split_whitespace();
    let Some(name) = words.next() else {
        return Ok(None);
    };
    let argument = words.next();
    let extra = words.next().is_some();

    let lowered = name.to_ascii_lowercase();

    // The `Enable*` family first, so it is matched by the same rule that generates its help.
    let subsystem = lowered.strip_prefix("enable").and_then(|rest| {
        Subsystem::ALL
            .into_iter()
            .find(|s| s.name().eq_ignore_ascii_case(rest))
    });

    let (spec_name, command) = match (subsystem, lowered.as_str()) {
        (Some(subsystem), _) => {
            let value = match argument {
                Some(text) => Some(parse_bool(text).ok_or_else(|| ParseError::NotABool {
                    value: text.to_string(),
                })?),
                None => None,
            };
            ("Enable*", Command::Enable(subsystem, value))
        }
        (None, "showstats") => {
            let value = match argument {
                Some(text) => Some(parse_bool(text).ok_or_else(|| ParseError::NotABool {
                    value: text.to_string(),
                })?),
                None => None,
            };
            ("ShowStats", Command::ShowStats(value))
        }
        (None, "help") | (None, "?") => ("Help", Command::Help),
        (None, "clear") | (None, "cls") => ("Clear", Command::Clear),
        (None, "stats") => ("Stats", Command::Stats),
        (None, _) => return Err(ParseError::Unknown(name.to_string())),
    };

    if extra {
        return Err(ParseError::TooManyArguments { name: spec_name });
    }

    Ok(Some(command))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_commands_parse_for_every_subsystem() {
        for subsystem in Subsystem::ALL {
            let line = format!("Enable{}", subsystem.name());
            assert_eq!(
                parse(&line),
                Ok(Some(Command::Enable(subsystem, None))),
                "{line} did not parse",
            );
            assert_eq!(
                parse(&format!("{line} false")),
                Ok(Some(Command::Enable(subsystem, Some(false)))),
            );
        }
    }

    /// Case is not part of a command name. Typing `enablesky` at 2am must work.
    #[test]
    fn names_and_arguments_are_case_insensitive() {
        assert_eq!(
            parse("ENABLESKY TRUE"),
            Ok(Some(Command::Enable(Subsystem::Sky, Some(true)))),
        );
        assert_eq!(
            parse("enableStaticMeshes Off"),
            Ok(Some(Command::Enable(Subsystem::StaticMeshes, Some(false)))),
        );
    }

    #[test]
    fn an_empty_line_is_not_an_error() {
        assert_eq!(parse(""), Ok(None));
        assert_eq!(parse("   \t "), Ok(None));
    }

    #[test]
    fn a_bad_line_says_what_is_wrong() {
        assert_eq!(
            parse("EnableFlying"),
            Err(ParseError::Unknown("EnableFlying".to_string())),
        );
        assert_eq!(
            parse("EnableSky maybe"),
            Err(ParseError::NotABool {
                value: "maybe".to_string(),
            }),
        );
        assert_eq!(
            parse("EnableSky true false"),
            Err(ParseError::TooManyArguments { name: "Enable*" }),
        );
    }

    /// `Help` is generated from the same table the parser reads, so this checks the property
    /// that makes that worth doing: everything listed is something you can actually type.
    #[test]
    fn every_line_help_prints_is_a_command_that_parses() {
        for line in help_lines() {
            let Some(usage) = line.strip_prefix("  ") else {
                continue;
            };
            let Some(name) = usage.split_whitespace().next() else {
                continue;
            };
            assert!(
                matches!(parse(name), Ok(Some(_))),
                "Help lists `{name}`, which does not parse",
            );
        }
    }
}
