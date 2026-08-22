//! The developer console — a toggleable overlay, an input line, scrollback, and commands
//! (AGENTS.md §13.5, §13.7 step 4).
//!
//! **Rudimentary on purpose.** Not a reproduction of Fable's `NGlobalConsole` — retail's console
//! is stripped and §3.9 rules out chasing it — and not a UI framework. History search,
//! autocomplete, coloured spans, resizing and theming are out of scope until something needs
//! them.
//!
//! Same shape as [`crate::scene`] and [`crate::text`], for the reason §11.1 gives: real input in,
//! plain renderer input out. Nothing here calls wgpu. [`Console::layout`] turns the console's
//! state into rectangles and positioned glyphs, [`draw`] turns those into
//! `renderer::GlyphInstance`s, and the split is what lets every rule below — wrapping, the
//! visible window, cursor movement, what a command prints — be tested with no GPU and no Fable
//! install (§6.9).
//!
//! **It draws nothing until it is opened.** `encode` is shared by `render` and
//! `render_to_image`, so anything on screen by default would land in `--screenshot` and break the
//! byte-identical capture that is this project's verification method (§13.6 prerequisite 3).

mod command;
mod draw;

pub use self::command::{Command, Subsystem};
pub use self::draw::draw;

use crate::text::layout::PlacedGlyph;
use crate::text::{Font, font::FontError, layout_line, layout_lines};
use std::collections::VecDeque;

/// The console's text size at a scale factor of 1, in pixels.
///
/// A convention, not a derivation — §2 exempts §13 from rule 1. 16 px is the size §13.1's
/// metrics are quoted at, where Inconsolata's cell is 8 × 17.
const BASE_PX: u32 = 16;

/// Fraction of the window height the console panel covers when open.
const PANEL_FRACTION: f32 = 0.45;

/// Inset from the panel's edges to its text, in unscaled pixels.
const PADDING: f32 = 6.0;

/// How many lines of output are kept. Old lines fall off the top.
const SCROLLBACK_LIMIT: usize = 512;

/// What the input line is prefixed with. Two characters, so it is one cell of gap.
const PROMPT: &str = "] ";

const PANEL_COLOUR: [f32; 4] = [0.04, 0.04, 0.06, 0.85];
const TEXT_COLOUR: [f32; 4] = [0.93, 0.95, 0.90, 1.0];
const CURSOR_COLOUR: [f32; 4] = [0.95, 0.78, 0.32, 1.0];

/// One key press, already translated out of the windowing system.
///
/// [`Input::Text`] carries *text*, not a key code: the platform has already applied the keyboard
/// layout and any modifier, which is the only way `Shift+3` types `#` on one layout and `£` on
/// another. `main.rs` reads it from `winit`'s `KeyEvent::text`; everything else here is a
/// physical key, because its meaning does not depend on the layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// The backquote/tilde key: open or close. Handled before [`Input::Text`] so the key that
    /// opens the console does not also type a backquote into it.
    Toggle,
    /// Close, if open. Escape.
    Close,
    Text(String),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    /// Older, then newer, entries from the command history.
    HistoryPrev,
    HistoryNext,
    /// Move the visible window of scrollback.
    ScrollUp,
    ScrollDown,
}

/// Something the console cannot do itself and hands back to the app.
///
/// The console owns everything that is console state — the scrollback, the stats overlay — and
/// performs those commands directly. What is left is what needs the renderer, which the console
/// deliberately cannot reach (§11.1). The app performs it and reports back with
/// [`Console::println`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// `Enable<Subsystem> [true|false]`. `None` means flip whatever it is now.
    Enable(Subsystem, Option<bool>),
    /// `Stats` — print what the renderer is holding.
    Stats,
}

/// A filled screen-space rectangle: the panel behind the text, and the cursor.
///
/// Drawn through the same instanced glyph pass as the text, pointing at the bindless array's
/// 1×1 white slot (`renderer::Renderer::solid_index`) — no second pipeline and no upload.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Fill {
    /// `(x, y, width, height)` in physical pixels, origin top-left.
    pub rect: [f32; 4],
    pub colour: [f32; 4],
}

/// One positioned character and the colour to draw it in.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColouredGlyph {
    pub placed: PlacedGlyph,
    pub colour: [f32; 4],
}

/// Everything to draw this frame, in back-to-front order: fills, then glyphs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub fills: Vec<Fill>,
    pub glyphs: Vec<ColouredGlyph>,
}

impl Layout {
    pub fn is_empty(&self) -> bool {
        self.fills.is_empty() && self.glyphs.is_empty()
    }
}

pub struct Console {
    font: Font,
    open: bool,
    /// The line being typed, and the cursor's **byte** offset into it.
    input: String,
    cursor: usize,
    scrollback: VecDeque<String>,
    /// Submitted lines, oldest first. `history_pos` is where Up/Down currently sits.
    history: Vec<String>,
    history_pos: Option<usize>,
    /// Wrapped lines scrolled up from the bottom. Reset to 0 whenever something is printed, so
    /// new output is never hidden.
    scroll: usize,
    /// `ShowStats`: the always-on overlay, off by default so `--screenshot` is unaffected.
    show_stats: bool,
    /// What that overlay says. The app refills it each frame — the console has no camera and no
    /// renderer to ask.
    stats: Vec<String>,
    /// The window's DPI scale factor, so text is the same physical size on a 4K display
    /// (§13.6's open DPI question). A change produces a different `px`, which is already a
    /// distinct cache key, so nothing else has to handle it (§13.3).
    scale: f32,
}

impl Console {
    /// Loads the embedded font. The only failure is a corrupt binary, and the app runs without a
    /// console rather than refusing to start.
    pub fn new() -> Result<Self, FontError> {
        Ok(Self {
            font: Font::new()?,
            open: false,
            input: String::new(),
            cursor: 0,
            scrollback: VecDeque::new(),
            history: Vec::new(),
            history_pos: None,
            scroll: 0,
            show_stats: false,
            stats: Vec::new(),
            scale: 1.0,
        })
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Whether the console is taking keystrokes — which is also the answer to "should the camera
    /// move?". One question with one answer, so a key cannot both type a `w` and fly forward.
    pub fn captures_input(&self) -> bool {
        self.open
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
    }

    /// Print into the scrollback. Embedded newlines become separate lines, so a multi-line
    /// message needs no ceremony at the call site.
    pub fn println(&mut self, text: impl AsRef<str>) {
        for line in text.as_ref().split('\n') {
            self.scrollback.push_back(line.to_string());
        }
        while self.scrollback.len() > SCROLLBACK_LIMIT {
            self.scrollback.pop_front();
        }
        // New output is never hidden behind wherever the view happened to be scrolled.
        self.scroll = 0;
    }

    pub fn show_stats(&self) -> bool {
        self.show_stats
    }

    /// Replace what the stats overlay says. Cheap to call every frame; it lays out only when
    /// `ShowStats` is on.
    pub fn set_stats(&mut self, lines: Vec<String>) {
        self.stats = lines;
    }

    /// Handle one key. Returns what the app must do, if anything.
    ///
    /// Every key except [`Input::Toggle`] is ignored while closed — the console is not a hidden
    /// keylogger, and the app is free to hand it everything.
    pub fn handle(&mut self, input: Input) -> Option<Effect> {
        if let Input::Toggle = input {
            self.open = !self.open;
            self.scroll = 0;
            return None;
        }
        if !self.open {
            return None;
        }

        match input {
            Input::Toggle => unreachable!("handled above"),
            Input::Close => self.open = false,
            Input::Text(text) => {
                // Control characters are not text, whatever the platform reported. Enter and
                // Backspace arrive as their own inputs and would otherwise be typed twice.
                for ch in text.chars().filter(|c| !c.is_control()) {
                    self.input.insert(self.cursor, ch);
                    self.cursor += ch.len_utf8();
                }
            }
            Input::Enter => return self.submit(),
            Input::Backspace => {
                let previous = self.previous_boundary();
                if previous < self.cursor {
                    self.input.replace_range(previous..self.cursor, "");
                    self.cursor = previous;
                }
            }
            Input::Delete => {
                let next = self.next_boundary();
                if next > self.cursor {
                    self.input.replace_range(self.cursor..next, "");
                }
            }
            Input::Left => self.cursor = self.previous_boundary(),
            Input::Right => self.cursor = self.next_boundary(),
            Input::Home => self.cursor = 0,
            Input::End => self.cursor = self.input.len(),
            Input::HistoryPrev => self.recall(true),
            Input::HistoryNext => self.recall(false),
            Input::ScrollUp => self.scroll += 1,
            Input::ScrollDown => self.scroll = self.scroll.saturating_sub(1),
        }

        None
    }

    /// Run a line as if it had been typed at the prompt, open or not.
    ///
    /// This is what `--command` goes through, so a scripted line and a typed one cannot take
    /// different paths — including the echo, the history and the error message.
    pub fn run_line(&mut self, line: &str) -> Option<Effect> {
        self.input = line.to_string();
        self.cursor = self.input.len();
        self.submit()
    }

    /// Echo the line, run it, and clear the prompt.
    fn submit(&mut self) -> Option<Effect> {
        let line = std::mem::take(&mut self.input);
        self.cursor = 0;
        self.history_pos = None;
        self.println(format!("{PROMPT}{line}"));

        if !line.trim().is_empty() && self.history.last().map(String::as_str) != Some(line.as_str())
        {
            self.history.push(line.clone());
        }

        match command::parse(&line) {
            Err(error) => {
                self.println(error.to_string());
                None
            }
            Ok(None) => None,
            Ok(Some(command)) => self.run(command),
        }
    }

    /// Perform what the console owns; hand back what it does not.
    fn run(&mut self, command: Command) -> Option<Effect> {
        match command {
            Command::Help => {
                for line in command::help_lines() {
                    self.println(line);
                }
                None
            }
            Command::Clear => {
                self.scrollback.clear();
                self.scroll = 0;
                None
            }
            Command::ShowStats(value) => {
                self.show_stats = value.unwrap_or(!self.show_stats);
                self.println(format!("ShowStats {}", self.show_stats));
                None
            }
            Command::Stats => Some(Effect::Stats),
            Command::Enable(subsystem, value) => Some(Effect::Enable(subsystem, value)),
        }
    }

    /// Walk the history. Stepping past the newest entry returns to the empty prompt, so there is
    /// always a way back to a blank line.
    fn recall(&mut self, older: bool) {
        if self.history.is_empty() {
            return;
        }
        self.history_pos = match (self.history_pos, older) {
            (None, true) => Some(self.history.len() - 1),
            (None, false) => None,
            (Some(0), true) => Some(0),
            (Some(index), true) => Some(index - 1),
            (Some(index), false) if index + 1 < self.history.len() => Some(index + 1),
            (Some(_), false) => None,
        };
        self.input = match self.history_pos {
            Some(index) => self.history[index].clone(),
            None => String::new(),
        };
        self.cursor = self.input.len();
    }

    fn previous_boundary(&self) -> usize {
        self.input[..self.cursor]
            .chars()
            .next_back()
            .map_or(0, |ch| self.cursor - ch.len_utf8())
    }

    fn next_boundary(&self) -> usize {
        self.input[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |ch| self.cursor + ch.len_utf8())
    }

    /// The text size in physical pixels at the current DPI scale.
    fn px(&self) -> u32 {
        ((BASE_PX as f32 * self.scale).round() as u32).max(8)
    }

    /// Turn the current state into fills and positioned glyphs for a `size`-pixel target.
    ///
    /// Returns an empty layout when the console is closed and the stats overlay is off — which
    /// is the default, and what keeps `--screenshot` byte-identical (§13.6).
    pub fn layout(&self, size: [u32; 2]) -> Layout {
        let mut layout = Layout::default();
        let (width, height) = (size[0].max(1) as f32, size[1].max(1) as f32);
        let px = self.px();
        let (cell_width, line_height) = self.font.cell(px);
        let padding = (PADDING * self.scale).round();

        let panel_height = if self.open {
            let raw = (height * PANEL_FRACTION).round();
            // Always room for the input line plus one line of output, however short the window.
            raw.max(padding * 2.0 + line_height * 2.0).min(height)
        } else {
            0.0
        };

        if self.open {
            layout.fills.push(Fill {
                rect: [0.0, 0.0, width, panel_height],
                colour: PANEL_COLOUR,
            });

            let columns = (((width - padding * 2.0) / cell_width).floor() as usize).max(1);
            let rows = (((panel_height - padding * 2.0) / line_height).floor() as usize).max(2);

            // The bottom row is the prompt; everything above it is scrollback.
            let input_top = padding + (rows - 1) as f32 * line_height;
            let output_rows = rows - 1;

            let wrapped: Vec<&str> = self
                .scrollback
                .iter()
                .flat_map(|line| wrap(line, columns))
                .collect();
            // `scroll` counts wrapped lines up from the bottom, and cannot go past the top.
            let hidden = wrapped.len().saturating_sub(output_rows);
            let first = hidden - self.scroll.min(hidden);

            // Bottom-aligned, like a terminal: with less output than rows, the gap belongs above
            // the text, not between the newest line and the prompt.
            let shown = wrapped[first..].len().min(output_rows);
            let top = padding + (output_rows - shown) as f32 * line_height;
            layout_lines(
                &self.font,
                px,
                wrapped[first..].iter().take(output_rows),
                [padding, top],
                &mut Placed(&mut layout.glyphs, TEXT_COLOUR),
            );

            // The prompt, horizontally scrolled so the cursor stays on screen.
            let cursor_column = PROMPT.chars().count() + self.input[..self.cursor].chars().count();
            let visible = columns.max(1);
            let offset = (cursor_column + 1).saturating_sub(visible);
            let line: String = format!("{PROMPT}{}", self.input)
                .chars()
                .skip(offset)
                .take(visible)
                .collect();
            layout_line(
                &self.font,
                px,
                &line,
                [padding, input_top],
                &mut Placed(&mut layout.glyphs, TEXT_COLOUR),
            );

            // An underline rather than a block: a block would hide the character under it, and
            // there are no per-glyph background colours to invert with (§13.5 — no rich text).
            let thickness = (2.0 * self.scale).round().max(1.0);
            layout.fills.push(Fill {
                rect: [
                    padding + (cursor_column - offset) as f32 * cell_width,
                    input_top + line_height - thickness,
                    cell_width,
                    thickness,
                ],
                colour: CURSOR_COLOUR,
            });
        }

        // The stats overlay sits under the console panel when both are up, rather than behind
        // it. Its own panel, so it stays readable over a bright sky without drawing every glyph
        // twice as a drop shadow.
        if self.show_stats && !self.stats.is_empty() {
            let columns = self
                .stats
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0) as f32;
            let top = panel_height + padding;
            layout.fills.push(Fill {
                rect: [
                    padding,
                    top,
                    columns * cell_width + padding * 2.0,
                    self.stats.len() as f32 * line_height + padding * 2.0,
                ],
                colour: PANEL_COLOUR,
            });
            layout_lines(
                &self.font,
                px,
                &self.stats,
                [padding * 2.0, top + padding],
                &mut Placed(&mut layout.glyphs, TEXT_COLOUR),
            );
        }

        layout
    }
}

/// Adapts [`layout_line`]'s `Vec<PlacedGlyph>` sink to this module's coloured one, so the pen
/// stays the only thing that knows where a character goes.
struct Placed<'a>(&'a mut Vec<ColouredGlyph>, [f32; 4]);

impl Extend<PlacedGlyph> for Placed<'_> {
    fn extend<I: IntoIterator<Item = PlacedGlyph>>(&mut self, iter: I) {
        let colour = self.1;
        self.0
            .extend(iter.into_iter().map(|placed| ColouredGlyph { placed, colour }));
    }
}

/// Break a line into chunks of at most `columns` characters.
///
/// A hard break, not a word wrap: the console's output is paths, numbers and identifiers rather
/// than prose, and a rule that always breaks in the same place is one that can be reasoned about
/// from the column count alone.
fn wrap(line: &str, columns: usize) -> Vec<&str> {
    if line.chars().count() <= columns {
        return vec![line];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut count = 0;
    for (offset, _) in line.char_indices() {
        if count == columns {
            chunks.push(&line[start..offset]);
            start = offset;
            count = 0;
        }
        count += 1;
    }
    chunks.push(&line[start..]);
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: [u32; 2] = [640, 480];

    fn console() -> Console {
        Console::new().expect("the embedded font parses")
    }

    fn type_line(console: &mut Console, line: &str) -> Option<Effect> {
        console.handle(Input::Text(line.to_string()));
        console.handle(Input::Enter)
    }

    /// The invariant §13.6 prerequisite 3 names: a closed console draws **nothing**, so
    /// `--screenshot` keeps producing the same bytes it did before there was a console.
    #[test]
    fn a_closed_console_lays_out_nothing() {
        let mut console = console();
        console.println("a line that is in the scrollback but not on screen");
        assert!(console.layout(SIZE).is_empty());

        console.handle(Input::Toggle);
        assert!(!console.layout(SIZE).is_empty(), "an open console draws");

        console.handle(Input::Toggle);
        assert!(console.layout(SIZE).is_empty(), "closing it puts it back");
    }

    /// The key that opens the console must not also type a backquote into it — the platform
    /// reports both for one press, and `main.rs` relies on `Toggle` being handled first.
    #[test]
    fn keys_do_nothing_while_closed_and_the_toggle_key_is_not_typed() {
        let mut console = console();
        console.handle(Input::Text("hello".to_string()));
        console.handle(Input::Enter);
        assert!(console.scrollback.is_empty(), "a closed console took input");

        console.handle(Input::Toggle);
        assert_eq!(console.input, "", "the toggle key typed itself");
    }

    #[test]
    fn typing_editing_and_submitting() {
        let mut console = console();
        console.handle(Input::Toggle);

        console.handle(Input::Text("Helo".to_string()));
        console.handle(Input::Left);
        console.handle(Input::Text("l".to_string()));
        assert_eq!(console.input, "Hello");

        console.handle(Input::End);
        console.handle(Input::Backspace);
        assert_eq!(console.input, "Hell");
        console.handle(Input::Home);
        console.handle(Input::Delete);
        assert_eq!(console.input, "ell");

        console.handle(Input::Enter);
        assert_eq!(console.input, "", "the prompt was not cleared");
        assert_eq!(console.scrollback[0], "] ell", "the line was not echoed");
        assert!(
            console.scrollback[1].contains("unknown command"),
            "a bad command said nothing: {:?}",
            console.scrollback[1],
        );
    }

    /// A control character reported as `text` — Enter arrives as `"\r"` on some platforms — must
    /// not be typed, or every submitted line would end in a stray glyph.
    #[test]
    fn control_characters_are_not_text() {
        let mut console = console();
        console.handle(Input::Toggle);
        console.handle(Input::Text("a\r\n\u{8}b".to_string()));
        assert_eq!(console.input, "ab");
    }

    #[test]
    fn history_walks_back_and_forward_to_an_empty_prompt() {
        let mut console = console();
        console.handle(Input::Toggle);
        type_line(&mut console, "Help");
        type_line(&mut console, "Clear");

        console.handle(Input::HistoryPrev);
        assert_eq!(console.input, "Clear");
        console.handle(Input::HistoryPrev);
        assert_eq!(console.input, "Help");
        console.handle(Input::HistoryPrev);
        assert_eq!(console.input, "Help", "walking past the oldest entry");
        console.handle(Input::HistoryNext);
        assert_eq!(console.input, "Clear");
        console.handle(Input::HistoryNext);
        assert_eq!(console.input, "", "there is always a way back to a blank line");
    }

    /// The console performs what it owns and hands back what needs the renderer, which it
    /// deliberately cannot reach (§11.1).
    #[test]
    fn commands_split_into_what_the_console_owns_and_what_it_hands_back() {
        let mut console = console();
        console.handle(Input::Toggle);

        assert_eq!(
            type_line(&mut console, "EnableLandscape false"),
            Some(Effect::Enable(Subsystem::Landscape, Some(false))),
        );
        assert_eq!(type_line(&mut console, "Stats"), Some(Effect::Stats));

        // ShowStats is console state, so it is performed here and reported.
        assert_eq!(type_line(&mut console, "ShowStats"), None);
        assert!(console.show_stats());
        assert_eq!(console.scrollback.back().map(String::as_str), Some("ShowStats true"));

        // So is Clear, and it empties the scrollback including its own echo.
        assert_eq!(type_line(&mut console, "Clear"), None);
        assert!(console.scrollback.is_empty());
    }

    #[test]
    fn scrollback_is_bounded_and_multi_line_output_splits() {
        let mut console = console();
        console.println("one\ntwo");
        assert_eq!(console.scrollback.len(), 2);

        for i in 0..SCROLLBACK_LIMIT + 10 {
            console.println(format!("line {i}"));
        }
        assert_eq!(console.scrollback.len(), SCROLLBACK_LIMIT);
        assert_eq!(
            console.scrollback.back().map(String::as_str),
            Some(format!("line {}", SCROLLBACK_LIMIT + 9).as_str()),
            "the newest line fell off instead of the oldest",
        );
    }

    #[test]
    fn wrapping_breaks_at_the_column_count() {
        assert_eq!(wrap("abcdef", 3), vec!["abc", "def"]);
        assert_eq!(wrap("abcd", 3), vec!["abc", "d"]);
        assert_eq!(wrap("abc", 3), vec!["abc"]);
        assert_eq!(wrap("", 3), vec![""], "an empty line still takes a row");
        // Char boundaries, not byte ones.
        assert_eq!(wrap("äöü", 2), vec!["äö", "ü"]);
    }

    /// Output sits against the prompt, like a terminal: a console with two lines in it does not
    /// leave a hole between them and the input line.
    #[test]
    fn short_output_is_bottom_aligned_against_the_prompt() {
        let mut console = console();
        console.handle(Input::Toggle);
        console.println("only line");

        let layout = console.layout(SIZE);
        let (_, line_height) = console.font.cell(console.px());
        let output = layout.glyphs[0].placed.baseline_y;
        let prompt = layout.glyphs.last().expect("the prompt row").placed.baseline_y;
        assert!(
            (prompt - output - line_height).abs() < 0.5,
            "the only output line is {} above the prompt, not one row",
            prompt - output,
        );
    }

    /// Scrollback shows the newest lines, and `ScrollUp` moves the window without ever running
    /// off either end.
    #[test]
    fn the_visible_window_holds_the_newest_lines_and_scrolls() {
        let mut console = console();
        console.handle(Input::Toggle);
        for i in 0..200 {
            console.println(format!("line {i}"));
        }

        let text_of = |layout: &Layout| -> Vec<char> {
            layout.glyphs.iter().map(|g| g.placed.key.ch).collect()
        };
        let bottom = text_of(&console.layout(SIZE));
        let joined: String = bottom.iter().collect();
        assert!(joined.contains("line 199"), "the newest line is not visible");

        console.handle(Input::ScrollUp);
        let scrolled: String = text_of(&console.layout(SIZE)).iter().collect();
        assert!(!scrolled.contains("line 199"), "ScrollUp did not move the window");

        for _ in 0..1000 {
            console.handle(Input::ScrollUp);
        }
        let top: String = text_of(&console.layout(SIZE)).iter().collect();
        assert!(top.contains("line 0"), "scrolling ran off the top: {top:?}");

        // Printing brings the view back to the bottom, so new output is never hidden.
        console.println("something happened");
        let after: String = text_of(&console.layout(SIZE)).iter().collect();
        assert!(after.contains("something happened"));
    }

    /// The stats overlay is the one thing that draws with the console closed, and only after
    /// `ShowStats` — which is why it cannot affect the default `--screenshot`.
    #[test]
    fn the_stats_overlay_draws_only_when_asked() {
        let mut console = console();
        console.set_stats(vec!["fps 60".to_string()]);
        assert!(console.layout(SIZE).is_empty());

        console.handle(Input::Toggle);
        type_line(&mut console, "ShowStats true");
        console.handle(Input::Toggle);

        let layout = console.layout(SIZE);
        let text: String = layout.glyphs.iter().map(|g| g.placed.key.ch).collect();
        assert!(text.contains("fps 60"), "the overlay drew {text:?}");
    }

    /// DPI: the same console at twice the scale factor is twice the size, and asks the cache for
    /// a different `px` — which is already a distinct key, so nothing else handles it (§13.6).
    #[test]
    fn the_scale_factor_reaches_the_glyph_size_and_the_geometry() {
        let mut console = console();
        console.handle(Input::Toggle);
        console.println("hello");

        let one = console.layout(SIZE);
        console.set_scale(2.0);
        let two = console.layout(SIZE);

        assert_eq!(one.glyphs[0].placed.key.px * 2, two.glyphs[0].placed.key.px);

        // And the geometry follows the glyph, not just the cache key: the cell is twice as wide,
        // so consecutive characters on a row are twice as far apart.
        let advance = |layout: &Layout| layout.glyphs[1].placed.pen_x - layout.glyphs[0].placed.pen_x;
        assert!(
            (advance(&two) - advance(&one) * 2.0).abs() < 0.5,
            "the cell went from {} to {}, not double",
            advance(&one),
            advance(&two),
        );
        // A nonsense scale factor is ignored rather than dividing by zero somewhere later.
        console.set_scale(0.0);
        assert_eq!(console.px(), BASE_PX);
    }

    /// A long line must not push the cursor off the panel: the prompt scrolls horizontally so
    /// the cursor is always on screen.
    #[test]
    fn a_long_prompt_scrolls_to_keep_the_cursor_visible() {
        let mut console = console();
        console.handle(Input::Toggle);
        console.handle(Input::Text("x".repeat(400)));

        let layout = console.layout(SIZE);
        let cursor = layout.fills.last().expect("the cursor is the last fill");
        assert!(
            cursor.rect[0] >= 0.0 && cursor.rect[0] + cursor.rect[2] <= SIZE[0] as f32,
            "the cursor is off screen at {:?}",
            cursor.rect,
        );
    }
}
