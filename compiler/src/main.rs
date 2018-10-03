use std::env;
use std::fs::File;
use std::io;

#[derive(Debug)]
enum Token {
    OpenBrace,
    CloseBrace,
    OpenParenthesis,
    CloseParenthesis,
    Semicolon,
    IntKeyword,
    ReturnKeyword,
    Identifier(String),
    Integer(u64),
}

#[derive(Debug)]
enum TokenizerState {
    ConsumeNextChar,
    ReadIdentifier(String),
    ReadInteger { number: u64, radix: Option<u32> },
}

struct Tokenizer {
    state: TokenizerState,
}

fn primitive_to_token(c: char) -> io::Result<Token> {
    use Token::*;
    let token = match c {
        '{' => OpenBrace,
        '}' => CloseBrace,
        '(' => OpenParenthesis,
        ')' => CloseParenthesis,
        ';' => Semicolon,
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unexpected character: {}", c)))?,
    };
    Ok(token)
}

fn string_to_token(s: &str) -> Token {
    use Token::*;
    match s {
        "Int" => IntKeyword,
        "Return" => ReturnKeyword,
        s => Identifier(s.to_string()),
    }
}

impl Tokenizer {
    fn new() -> Self {
        Tokenizer {
            state: TokenizerState::ConsumeNextChar,
        }
    }

    fn tokenize<S: io::BufRead>(&mut self, stream: S) -> io::Result<Vec<Token>> {
        use Token::*;
        use TokenizerState::*;

        let mut tokens: Vec<Token> = Vec::new();

        for line in stream.lines() {
            let line = line?;

            let mut chars = line.chars();

            while let Some(ch) = chars.next() {
                let mut new_state = None;

                match self.state {
                    ConsumeNextChar => {

                        match ch { 
                            '{'|'}'|'('|')'|';'=> {
                                tokens.push(primitive_to_token(ch)?);
                            }

                            '0' => {
                                new_state = Some(ReadInteger { number: 0, radix: None });
                            }

                            // It's ok to unwrap, we made sure it can only be 0..9
                            '1'...'9' => {
                                let radix = Some(10);
                                let number = ch.to_digit(10).unwrap() as u64;
                                new_state = Some(ReadInteger { number, radix });
                            }

                            ch if ch.is_alphabetic() || ch == '_' => {
                                new_state = Some(ReadIdentifier(ch.to_string()));
                            }

                            ch if ch.is_whitespace() => {}

                            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Encountered unknown character: {}", ch)))?,
                        }
                    }

                    ReadIdentifier(ref mut s) => {
                        match ch { 
                            '{'|'}'|'('|')'|';'=> {
                                tokens.push(string_to_token(s));
                                tokens.push(primitive_to_token(ch)?);
                                new_state = Some(ConsumeNextChar);
                            },

                            ch if ch.is_alphanumeric() || ch == '_' => { 
                                s.push(ch);
                            },

                            ch if ch.is_whitespace() => {
                                tokens.push(string_to_token(s));
                                new_state = Some(ConsumeNextChar);
                            },

                            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Encountered unknown character: {}", ch)))?,
                        }
                    }

                    ReadInteger { ref mut number, ref mut radix } => {
                        match ch { 
                            '{'|'}'|'('|')'|';'=> {
                                tokens.push(Integer( *number ));
                                tokens.push(primitive_to_token(ch)?);
                                new_state = Some(ConsumeNextChar);
                            }

                            '0'...'9' | 'a'...'f' | 'o' | 'x' => {
                                if let Some(radix) = *radix { 
                                    if let Some(digit) = ch.to_digit(radix) {
                                        *number = (radix as u64) * *number + digit as u64;
                                    } else {
                                        Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unexpected character: {}", ch)))?;
                                    }
                                } else {
                                    match ch {
                                        '0'...'9' => {
                                            *radix = Some(10);
                                            *number = 10 * *number + ch.to_digit(10).unwrap() as u64;
                                        },
                                        'b' => { *radix = Some(2); },
                                        'o' => { *radix = Some(8); },
                                        'x' => { *radix = Some(16); },
                                        ch => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unexpected character: {}", ch)))?,
                                    }
                                }
                            }

                            ch if ch.is_whitespace() => {
                                tokens.push(Integer(*number));
                                new_state = Some(ConsumeNextChar);
                            }

                            ch if ch.is_alphabetic() =>
                                Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unexpected character: {}", ch)))?,

                            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Encountered unknown character: {}", ch)))?,
                        }

                    }
                }

                if let Some(new_state) = new_state {
                    self.state = new_state;
                }
            }

            // End of line is reached, dump whatever state we are in
            match self.state {
                ReadIdentifier(ref s) => tokens.push(string_to_token(s)),
                ReadInteger { number, .. } => tokens.push(Integer(number)),
                _ => {}
            }

            self.state = ConsumeNextChar;
        }

        Ok(tokens)
    }
}

fn lex<S: io::BufRead>(stream: S) -> io::Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(stream)?
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    // executable
    args.next();
    let filename = args.next().expect("Need to specify filename");
    let file = File::open(filename)?;

    let buf_reader = io::BufReader::new(file);

    let tokens = lex(buf_reader)?;

    println!("{:?}", tokens);

    Ok(())
}
