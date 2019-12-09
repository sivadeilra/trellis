use core::iter::Peekable;
use core::ops::Range;
use core::str::CharIndices;

const UNICODE_BYTE_MARKER: char = '\u{FEFF}';

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Equals,
    Ident(String),
    String(String),
    Integer(i64),
    Float(f64),
    Number,
    Comma,
    Arrow, // ->
    Semicolon,
}

pub struct Location {
    pub range: Range<usize>,
}

pub struct LexerError {
    message: String,
    location: Location,
}

pub struct Lexer<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
    errors: Vec<LexerError>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.char_indices().peekable(),
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, start: usize, message: String) {
        let end = match self.chars.peek() {
            Some(&(end, _)) => end,
            None => self.text.len(),
        };

        self.errors.push(LexerError {
            message,
            location: Location { range: start..end },
        });
    }

    fn map_offset_to_line(&self, offset: usize) -> Result<(usize, usize), ()> {
        if offset > self.text.len() {
            // Out of range
            return Err(());
        }

        let mut line = 1;
        let mut column = 1;
        for (i, c) in self.text.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        Ok((line, column))
    }

    pub fn write_errors(&self, output: &mut dyn core::fmt::Write) -> core::fmt::Result {
        for error in self.errors.iter() {
            let line_column = self.map_offset_to_line(error.location.range.start);
            if let Ok((line, column)) = line_column {
                write!(output, "({}, {}): ", line, column)?;
            }
            write!(output, "{}\n", error.message)?;
        }
        Ok(())
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[LexerError] {
        &self.errors
    }

    fn consumec(&mut self) {
        self.chars.next();
    }

    fn nextc(&mut self) -> Option<char> {
        Some(self.chars.next()?.1)
    }
    fn peekc(&mut self) -> Option<char> {
        Some(self.chars.peek()?.1)
    }

    pub fn next(&mut self) -> Option<Token> {
        loop {
            let (start, c) = self.chars.next()?;
            match c {
                ' ' | '\r' | '\n' | '\t' => {
                    // Efficiently match the most common whitespace characters.
                    continue;
                }
                UNICODE_BYTE_MARKER => {}
                '=' => return Some(Token::Equals),
                '{' => return Some(Token::LCurly),
                '}' => return Some(Token::RCurly),
                '[' => return Some(Token::LSquare),
                ']' => return Some(Token::RSquare),
                ',' => return Some(Token::Comma),
                ';' => return Some(Token::Semicolon),
                '-' => {
                    if self.peekc() == Some('>') {
                        self.consumec();
                        return Some(Token::Arrow);
                    }
                }
                '\"' => {
                    return self.parse_quoted_string(start);
                }
                '_' => {
                    return self.parse_ident(start, c);
                }
                c => {
                    // [a-z][A-Z] starts an identifier.
                    if c.is_ascii_alphabetic() {
                        return self.parse_ident(start, c);
                    }

                    // [0-9] starts a number.
                    if c.is_ascii_digit() {
                        return self.parse_number(start, c);
                    }

                    // Use the more expensive Unicode definition of is_whitespace().
                    if c.is_whitespace() {
                        continue;
                    }

                    // The character is not recognized.
                    // We'll report the error, below.
                }
            }

            // The character is not recognized.
            self.add_error(start, format!("Unrecognized token."));
            return None;
        }
    }

    fn parse_ident(&mut self, _start: usize, c: char) -> Option<Token> {
        let mut ident = String::new();
        ident.push(c);
        while let Some(c) = self.peekc() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.chars.next();
                ident.push(c);
            } else {
                break;
            }
        }
        return Some(Token::Ident(ident));
    }

    fn parse_quoted_string(&mut self, start: usize) -> Option<Token> {
        let mut s = String::new();
        let mut terminated = false;
        while let Some(c) = self.nextc() {
            match c {
                '\"' => {
                    terminated = true;
                    break;
                }
                '\\' => {
                    // Process an escape sequence.
                    match self.nextc() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some(c) => {
                            // Unrecognized escape sequence.
                            s.push('\\');
                            s.push(c);
                        }
                        None => break,
                    }
                }
                c => s.push(c),
            }
        }
        if !terminated {
            self.add_error(start, format!("Quoted string was not terminated."));
            return None;
        }
        s.shrink_to_fit();
        return Some(Token::String(s));
    }

    fn parse_number(&mut self, start: usize, c: char) -> Option<Token> {
        let mut buf = String::new();
        buf.push(c);
        while let Some(c) = self.peekc() {
            if c.is_ascii_digit() {
                self.consumec();
                buf.push(c);
                continue;
            }
            if c == '.' {
                // Could be a floating-point number.
                let mut ic = self.chars.clone();
                ic.next(); // consume '.' (in the cloned iterator)
                if let Some(&(_, c)) = ic.peek() {
                    if c.is_ascii_digit() {
                        // It looks like this is a floating point number.
                        // Commit to that.
                        self.consumec(); // consume '.'
                        buf.push('.');
                        while let Some(c) = self.peekc() {
                            if c.is_ascii_digit() {
                                self.consumec();
                                buf.push(c);
                            } else {
                                break;
                            }
                        }
                    } else {
                        // Not followed by a digit, so this is not a Float.
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Convert the number.
        if buf.contains('.') {
            if let Ok(number_value) = buf.parse::<f64>() {
                return Some(Token::Float(number_value));
            } else {
                self.add_error(start, format!("Failed to parse floating-point value."));
                return None;
            }
        } else {
            if let Ok(number_value) = buf.parse::<i64>() {
                return Some(Token::Integer(number_value));
            } else {
                self.add_error(start, format!("Failed to parse integer."));
                // report error
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        fn case(input: &str, expected: Vec<Token>) {
            println!("case: input = {:?}", input);
            let mut lexer = Lexer::new(input);
            let tokens: Vec<Token> = lexer.by_ref().collect();
            if lexer.has_errors() {
                let mut s = String::new();
                lexer.write_errors(&mut s).unwrap();
                panic!("lexer reported errors: {}", s);
            }

            println!("tokens = {:?}", tokens);
            assert_eq!(tokens, expected);
        }

        fn case_err(input: &str, expected: Vec<Token>, _expected_errors: &str) {
            println!("case: input = {:?}", input);
            let mut lexer = Lexer::new(input);
            let tokens: Vec<Token> = lexer.by_ref().collect();
            if lexer.has_errors() {
                let mut s = String::new();
                lexer.write_errors(&mut s).unwrap();
                println!("lexer reported errors: {}", s);
            }

            println!("tokens = {:?}", tokens);
            assert_eq!(tokens, expected);
        }

        case("", vec![]);
        case(" ", vec![]);
        case("\"foo\"", vec![Token::String("foo".to_string())]);

        case(
            "foo -> bar",
            vec![
                Token::Ident("foo".to_string()),
                Token::Arrow,
                Token::Ident("bar".to_string()),
            ],
        );

        case(
            "digraph { mumble \"frotz\" = [sides=17] }",
            vec![
                Token::Ident("digraph".to_string()),
                Token::LCurly,
                Token::Ident("mumble".to_string()),
                Token::String("frotz".to_string()),
                Token::Equals,
                Token::LSquare,
                Token::Ident("sides".to_string()),
                Token::Equals,
                Token::Integer(17),
                Token::RSquare,
                Token::RCurly,
            ],
        );

        case_err("\"unterminated string", vec![], "");
    }

    fn test_lexer(text: &str) {
        let mut lexer = Lexer::new(text);
        for _token in lexer.by_ref() {
            // println!("{:?}", token);
        }
        if lexer.has_errors() {
            println!("Uh oh!  Errors:\n");
            let mut s = String::new();
            lexer.write_errors(&mut s).unwrap();
            println!("{}", s);
        }
    }

    #[test]
    fn test_crazy() {
        test_lexer(r##"
digraph "unix" {
	graph [	fontname = "Helvetica-Oblique",
		fontsize = 36,
		label = "\n\n\n\nObject Oriented Graphs\nStephen North, 3/19/93",
		size = "6,6" ];
	node [	shape = polygon,
		sides = 4,
		distortion = "0.0",
		orientation = "0.0",
		skew = "0.0",
		color = white,
		style = filled,
		fontname = "Helvetica-Outline" ];
	"5th Edition" [sides=9, distortion="0.936354", orientation=28, skew="-0.126818", color=salmon2];
	"6th Edition" [sides=5, distortion="0.238792", orientation=11, skew="0.995935", color=deepskyblue];
	"PWB 1.0" [sides=8, distortion="0.019636", orientation=79, skew="-0.440424", color=goldenrod2];
	LSX [sides=9, distortion="-0.698271", orientation=22, skew="-0.195492", color=burlywood2];
	"1 BSD" [sides=7, distortion="0.265084", orientation=26, skew="0.403659", color=gold1];
	"Mini Unix" [distortion="0.039386", orientation=2, skew="-0.461120", color=greenyellow];
	Wollongong [sides=5, distortion="0.228564", orientation=63, skew="-0.062846", color=darkseagreen];
	Interdata [distortion="0.624013", orientation=56, skew="0.101396", color=dodgerblue1];
	"Unix/TS 3.0" [sides=8, distortion="0.731383", orientation=43, skew="-0.824612", color=thistle2];
	"PWB 2.0" [sides=6, distortion="0.592100", orientation=34, skew="-0.719269", color=darkolivegreen3];
	"7th Edition" [sides=10, distortion="0.298417", orientation=65, skew="0.310367", color=chocolate];
	"8th Edition" [distortion="-0.997093", orientation=50, skew="-0.061117", color=turquoise3];
	"32V" [sides=7, distortion="0.878516", orientation=19, skew="0.592905", color=steelblue3];
	V7M [sides=10, distortion="-0.960249", orientation=32, skew="0.460424", color=navy];
	"Ultrix-11" [sides=10, distortion="-0.633186", orientation=10, skew="0.333125", color=darkseagreen4];
	Xenix [sides=8, distortion="-0.337997", orientation=52, skew="-0.760726", color=coral];
	"UniPlus+" [sides=7, distortion="0.788483", orientation=39, skew="-0.526284", color=darkolivegreen3];
	"9th Edition" [sides=7, distortion="0.138690", orientation=55, skew="0.554049", color=coral3];
	"2 BSD" [sides=7, distortion="-0.010661", orientation=84, skew="0.179249", color=blanchedalmond];
	"2.8 BSD" [distortion="-0.239422", orientation=44, skew="0.053841", color=lightskyblue1];
	"2.9 BSD" [distortion="-0.843381", orientation=70, skew="-0.601395", color=aquamarine2];
	"3 BSD" [sides=10, distortion="0.251820", orientation=18, skew="-0.530618", color=lemonchiffon];
	"4 BSD" [sides=5, distortion="-0.772300", orientation=24, skew="-0.028475", color=darkorange1];
	"4.1 BSD" [distortion="-0.226170", orientation=38, skew="0.504053", color=lightyellow1];
	"4.2 BSD" [sides=10, distortion="-0.807349", orientation=50, skew="-0.908842", color=darkorchid4];
	"4.3 BSD" [sides=10, distortion="-0.030619", orientation=76, skew="0.985021", color=lemonchiffon2];
	"Ultrix-32" [distortion="-0.644209", orientation=21, skew="0.307836", color=goldenrod3];
	"PWB 1.2" [sides=7, distortion="0.640971", orientation=84, skew="-0.768455", color=cyan];
	"USG 1.0" [distortion="0.758942", orientation=42, skew="0.039886", color=blue];
	"CB Unix 1" [sides=9, distortion="-0.348692", orientation=42, skew="0.767058", color=firebrick];
	"USG 2.0" [distortion="0.748625", orientation=74, skew="-0.647656", color=chartreuse4];
	"CB Unix 2" [sides=10, distortion="0.851818", orientation=32, skew="-0.020120", color=greenyellow];
	"CB Unix 3" [sides=10, distortion="0.992237", orientation=29, skew="0.256102", color=bisque4];
	"Unix/TS++" [sides=6, distortion="0.545461", orientation=16, skew="0.313589", color=mistyrose2];
	"PDP-11 Sys V" [sides=9, distortion="-0.267769", orientation=40, skew="0.271226", color=cadetblue1];
	"USG 3.0" [distortion="-0.848455", orientation=44, skew="0.267152", color=bisque2];
	"Unix/TS 1.0" [distortion="0.305594", orientation=75, skew="0.070516", color=orangered];
	"TS 4.0" [sides=10, distortion="-0.641701", orientation=50, skew="-0.952502", color=crimson];
	"System V.0" [sides=9, distortion="0.021556", orientation=26, skew="-0.729938", color=darkorange1];
	"System V.2" [sides=6, distortion="0.985153", orientation=33, skew="-0.399752", color=darkolivegreen4];
	"System V.3" [sides=7, distortion="-0.687574", orientation=58, skew="-0.180116", color=lightsteelblue1];
	"5th Edition" -> "6th Edition";
	"5th Edition" -> "PWB 1.0";
	"6th Edition" -> LSX;
	"6th Edition" -> "1 BSD";
	"6th Edition" -> "Mini Unix";
	"6th Edition" -> Wollongong;
	"6th Edition" -> Interdata;
	Interdata -> "Unix/TS 3.0";
	Interdata -> "PWB 2.0";
	Interdata -> "7th Edition";
	"7th Edition" -> "8th Edition";
	"7th Edition" -> "32V";
	"7th Edition" -> V7M;
	"7th Edition" -> "Ultrix-11";
	"7th Edition" -> Xenix;
	"7th Edition" -> "UniPlus+";
	V7M -> "Ultrix-11";
	"8th Edition" -> "9th Edition";
	"1 BSD" -> "2 BSD";
	"2 BSD" -> "2.8 BSD";
	"2.8 BSD" -> "Ultrix-11";
	"2.8 BSD" -> "2.9 BSD";
	"32V" -> "3 BSD";
	"3 BSD" -> "4 BSD";
	"4 BSD" -> "4.1 BSD";
	"4.1 BSD" -> "4.2 BSD";
	"4.1 BSD" -> "2.8 BSD";
	"4.1 BSD" -> "8th Edition";
	"4.2 BSD" -> "4.3 BSD";
	"4.2 BSD" -> "Ultrix-32";
	"PWB 1.0" -> "PWB 1.2";
	"PWB 1.0" -> "USG 1.0";
	"PWB 1.2" -> "PWB 2.0";
	"USG 1.0" -> "CB Unix 1";
	"USG 1.0" -> "USG 2.0";
	"CB Unix 1" -> "CB Unix 2";
	"CB Unix 2" -> "CB Unix 3";
	"CB Unix 3" -> "Unix/TS++";
	"CB Unix 3" -> "PDP-11 Sys V";
	"USG 2.0" -> "USG 3.0";
	"USG 3.0" -> "Unix/TS 3.0";
	"PWB 2.0" -> "Unix/TS 3.0";
	"Unix/TS 1.0" -> "Unix/TS 3.0";
	"Unix/TS 3.0" -> "TS 4.0";
	"Unix/TS++" -> "TS 4.0";
	"CB Unix 3" -> "TS 4.0";
	"TS 4.0" -> "System V.0";
	"System V.0" -> "System V.2";
	"System V.2" -> "System V.3";
}
        "##);
    }
}
