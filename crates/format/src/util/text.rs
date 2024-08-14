use unicode_segmentation::UnicodeSegmentation;

/// This is a generic lexer that should work for all of Fable's text formats (fingers crossed)
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    grapheme_position: usize,
    grapheme_location: Location,
    token_location: Location,
    state: LexerState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LexerState {
    // Starting state, and when delimiters like separators or whitespace are encountered.
    Root,
    // State for identifiers such as keys, booleans, structure names, etc.
    Identifier,
    EnterString,
    ExitString,
    Number,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            grapheme_position: 0,
            grapheme_location: Default::default(),
            token_location: Default::default(),
            state: LexerState::Root,
        }
        // If it is a newline, reset the grapheme column and update the grapheme line.
        // If it is anything other than a new line, update the grapheme column.
    }

    pub fn next_token(&mut self) -> Result<Option<Token<'a>>, Location> {
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
                        if self.state == LexerState::Root {
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

    fn next_grapheme(&mut self) -> Result<Option<Token<'a>>, Location> {
        // Grab the next grapheme in the source, if there is one, else return that we're finished.
        let grapheme = match self.source[self.grapheme_position..].graphemes(true).next() {
            Some(grapheme) => grapheme,
            None => return Ok(None),
        };

        // Process the grapheme according to the state of the tokenizer, possibly producing a token.
        let maybe_token = match self.state {
            LexerState::Root => self.do_root(grapheme),
            LexerState::EnterString => self.do_enter_string(grapheme),
            LexerState::ExitString => self.do_exit_string(grapheme),
            LexerState::Number => self.do_number(grapheme),
            LexerState::Identifier => self.do_identifier(grapheme),
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

    fn do_root(&mut self, grapheme: &'a str) -> Result<Option<Token<'a>>, Location> {
        let maybe_token = match grapheme {
            // Whitespace
            " " | "\n" | "\r" | "\r\n" => {
                self.source = &self.source[grapheme.len()..];
                Some(Token::new(
                    TokenKind::Whitespace,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Opening double quotes
            "\"" => {
                self.source = &self.source[grapheme.len()..];
                self.state = LexerState::EnterString;
                Some(Token::new(
                    TokenKind::Symbol,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Symbol
            "(" | ")" | "[" | "]" | "." | "," | "-" | ";" => {
                self.source = &self.source[grapheme.len()..];
                self.state = LexerState::Root;
                Some(Token::new(
                    TokenKind::Symbol,
                    self.grapheme_location,
                    grapheme,
                ))
            }
            // Number
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                self.state = LexerState::Number;
                self.token_location = self.grapheme_location;
                None
            }
            // Other
            _ => {
                self.state = LexerState::Identifier;
                self.token_location = self.grapheme_location;
                None
            }
        };

        // Advance to the next grapheme.
        self.grapheme_position += grapheme.len();

        Ok(maybe_token)
    }

    fn do_enter_string(&mut self, grapheme: &'a str) -> Result<Option<Token<'a>>, Location> {
        Ok(match grapheme {
            // TODO: Determine whether string escaping is supported.
            // TODO: Determine whether newlines in strings are allowed.
            // Closing double quotes
            "\"" => {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = LexerState::ExitString;

                Some(Token::new(TokenKind::String, self.token_location, text))
            }
            _ => {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            }
        })
    }

    fn do_exit_string(&mut self, grapheme: &'a str) -> Result<Option<Token<'a>>, Location> {
        Ok(if grapheme == "\"" {
            self.source = &self.source[grapheme.len()..];
            self.state = LexerState::Root;
            Some(Token::new(
                TokenKind::Symbol,
                self.grapheme_location,
                grapheme,
            ))
        } else {
            None
        })
    }

    fn do_number(&mut self, grapheme: &'a str) -> Result<Option<Token<'a>>, Location> {
        Ok(match grapheme {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            }
            _ => {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = LexerState::Root;

                Some(Token::new(TokenKind::Number, self.token_location, text))
            }
        })
    }

    fn do_identifier(&mut self, grapheme: &'a str) -> Result<Option<Token<'a>>, Location> {
        Ok(
            if "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_".contains(grapheme)
            {
                // Advance to the next grapheme.
                self.grapheme_position += grapheme.len();
                None
            } else {
                let text = &self.source[..self.grapheme_position];

                self.source = &self.source[self.grapheme_position..];
                self.state = LexerState::Root;

                Some(Token::new(TokenKind::Identifier, self.token_location, text))
            },
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub location: Location,
    pub text: &'a str,
}

impl<'a> Token<'a> {
    fn new(kind: TokenKind, location: Location, text: &'a str) -> Self {
        Self {
            kind,
            location,
            text,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
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
