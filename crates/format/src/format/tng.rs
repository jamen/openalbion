use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub struct TngLexer<'a> {
    source: &'a str,
    grapheme_position: usize,
    grapheme_location: Location,
    token_location: Location,
    state: TngLexerState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TngLexerState {
    // Starting state, and when delimiters like separators or whitespace are encountered.
    Root,
    // State for identifiers such as keys, booleans, structure names, etc.
    Identifier,
    EnterString,
    ExitString,
    Number,
}

impl<'a> TngLexer<'a> {
    pub fn new(source: &'a str) -> Self {
        TngLexer {
            source,
            grapheme_position: 0,
            grapheme_location: Default::default(),
            token_location: Default::default(),
            state: TngLexerState::Root,
        }
    }

    pub fn next_token(&mut self) -> Result<Option<TngToken<'a>>, Location> {
        // Copy the current grapheme location as the token location
        // because the next token might consist of multiple graphemes.
        self.token_location = self.grapheme_location;

        loop {
            // println!("?");
            match self.next_grapheme() {
                Ok(Some(token)) => {
                    self.grapheme_position = 0;
                    return Ok(Some(token));
                }
                Ok(None) => {
                    if self.source.len() == 0 {
                        if self.state == TngLexerState::Root {
                            return Ok(None);
                        } else {
                            return Err(self.token_location);
                        }
                    }
                }
                Err(location) => return Err(location),
            }
        }
    }

    fn next_grapheme(&mut self) -> Result<Option<TngToken<'a>>, Location> {
        // Grab the next grapheme in the source, if there is one, else return that we're finished.
        let grapheme = match self.source[self.grapheme_position..].graphemes(true).next() {
            Some(grapheme) => grapheme,
            None => return Ok(None),
        };

        // Process the grapheme according to the state of the tokenizer, possibly producing a token.
        let maybe_token = match self.state {
            TngLexerState::Root => self.do_root(grapheme),
            TngLexerState::EnterString => self.do_enter_string(grapheme),
            TngLexerState::ExitString => self.do_exit_string(grapheme),
            TngLexerState::Number => self.do_number(grapheme),
            TngLexerState::Identifier => self.do_identifier(grapheme),
        };

        // Update line/col
        if grapheme == "\n" || grapheme == "\r" || grapheme == "\r\n" {
            self.grapheme_location.column = 0;
            self.grapheme_location.line += 1;
        } else {
            self.grapheme_location.column += 1;
        };

        maybe_token
    }

    fn do_root(&mut self, grapheme: &'a str) -> Result<Option<TngToken<'a>>, Location> {
        let maybe_token = match grapheme {
            // Whitespace
            " " | "\n" | "\r" | "\r\n" => {
                self.source = &self.source[grapheme.len()..];
                Some(TngToken::new(
                    TngTokenKind::Whitespace,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Opening double quotes
            "\"" => {
                self.source = &self.source[grapheme.len()..];
                self.state = TngLexerState::EnterString;
                Some(TngToken::new(
                    TngTokenKind::Symbol,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Symbol
            "(" | ")" | "[" | "]" | "." | "," | "-" | ";" => {
                self.source = &self.source[grapheme.len()..];
                self.state = TngLexerState::Root;
                Some(TngToken::new(
                    TngTokenKind::Symbol,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Number
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                self.state = TngLexerState::Number;
                self.token_location = self.grapheme_location;
                None
            }
            // Other
            _ => {
                self.state = TngLexerState::Identifier;
                self.token_location = self.grapheme_location;
                None
            }
        };

        // Advance to the next grapheme.
        self.grapheme_position += grapheme.len();

        Ok(maybe_token)
    }

    fn do_enter_string(&mut self, grapheme: &'a str) -> Result<Option<TngToken<'a>>, Location> {
        Ok(match grapheme {
            // TODO: Determine whether string escaping is supported.
            // TODO: Determine whether newlines in strings are allowed.
            // Closing double quotes
            "\"" => {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = TngLexerState::ExitString;

                Some(TngToken::new(
                    TngTokenKind::String,
                    self.token_location,
                    text,
                ))
            }
            _ => {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            }
        })
    }

    fn do_exit_string(&mut self, grapheme: &'a str) -> Result<Option<TngToken<'a>>, Location> {
        Ok(if grapheme == "\"" {
            self.source = &self.source[grapheme.len()..];
            self.state = TngLexerState::Root;
            Some(TngToken::new(
                TngTokenKind::Symbol,
                self.grapheme_location,
                grapheme,
            ))
        } else {
            None
        })
    }

    fn do_number(&mut self, grapheme: &'a str) -> Result<Option<TngToken<'a>>, Location> {
        Ok(match grapheme {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            }
            _ => {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = TngLexerState::Root;

                Some(TngToken::new(
                    TngTokenKind::Number,
                    self.token_location,
                    text,
                ))
            }
        })
    }

    fn do_identifier(&mut self, grapheme: &'a str) -> Result<Option<TngToken<'a>>, Location> {
        Ok(
            if "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_".contains(grapheme)
            {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            } else {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = TngLexerState::Root;

                Some(TngToken::new(
                    TngTokenKind::Identifier,
                    self.token_location,
                    text,
                ))
            },
        )
    }
}

#[derive(Clone, Debug)]
pub struct TngToken<'a> {
    pub kind: TngTokenKind,
    pub location: Location,
    pub text: &'a str,
}

impl<'a> TngToken<'a> {
    fn new(kind: TngTokenKind, location: Location, text: &'a str) -> Self {
        Self {
            kind,
            location,
            text,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TngTokenKind {
    Identifier,
    String,
    Number,
    Symbol,
    Whitespace,
}

// Location of a token, or unrecognized token upon error.
#[derive(Default, Copy, Clone, Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
