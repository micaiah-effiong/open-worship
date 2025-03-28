use std::usize;

use gtk::glib::char;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TokenEnum {
    NUMBER,
    IDENTIFIER,
    COLON,
    HYPHEN,
    COMMA,
    ILLEGAL,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub t_type: TokenEnum,
    pub value: String,
}

impl Token {
    pub fn inspect(&self) -> String {
        return self.value.clone();
    }
}

pub struct Tokenizer {
    pub char: char,
    pub position: u32,
    pub peek_position: u32,
    pub input: String,
}

impl Tokenizer {
    pub fn new(inp: String) -> Self {
        // pad input before initializing lexer
        let input: String;
        input = String::from("  ") + &inp;

        return Tokenizer {
            char: input.chars().nth(0).expect("Input cannot be empty"),
            position: 0,
            peek_position: 1,
            input,
        };
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_white_space();
        // println!("CHAR {}", self.char);

        let token = match self.char {
            ':' => Token {
                t_type: TokenEnum::COLON,
                value: String::from(":"),
            },
            '-' => Token {
                t_type: TokenEnum::HYPHEN,
                value: String::from("-"),
            },
            ',' => Token {
                t_type: TokenEnum::COMMA,
                value: String::from(","),
            },

            // ===
            // handle EOF
            // ===
            '\0' => Token {
                t_type: TokenEnum::EOF,
                value: String::from('\0'),
            },

            ch => {
                if Tokenizer::is_letter(ch) {
                    // read string
                    let str = self.read_string();
                    return Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: str,
                    };
                }

                if Tokenizer::is_digit(ch) {
                    // read digit
                    let num = self.read_digit();
                    return Token {
                        t_type: TokenEnum::NUMBER,
                        value: num,
                    };
                }

                Token {
                    value: String::from(self.char),
                    t_type: TokenEnum::ILLEGAL,
                }
            }
        };

        self.read_char();
        return token;
    }

    pub fn skip_white_space(&mut self) {
        while self.char.is_whitespace() {
            self.read_char();
        }
    }

    fn read_string(&mut self) -> String {
        let start = self.position;
        while Tokenizer::is_letter(self.char) {
            self.read_char();
        }

        return self
            .input
            .get(start as usize..self.position as usize)
            .unwrap() // TODO: handle error None arm
            .to_string();
    }

    fn read_digit(&mut self) -> String {
        let start = self.position;
        while Tokenizer::is_digit(self.char) {
            self.read_char();
        }

        return self
            .input
            .get(start as usize..self.position as usize)
            .unwrap() // TODO: handle error None arm
            .to_string();
    }

    pub fn read_char(&mut self) {
        if self.peek_position as usize >= self.input.len() {
            self.char = '\0'
        } else {
            match self.input.chars().nth(self.peek_position as usize) {
                Some(ch) => self.char = ch,
                None => (),
            };
        }

        self.position = self.peek_position;
        self.peek_position = self.peek_position + 1;
    }

    fn is_letter(ch: char) -> bool {
        ch.is_ascii_alphabetic()
    }

    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }
}

#[cfg(test)]
mod test {
    // use crate::parser::tokenizer::{Token, TokenEnum, Tokenizer};
    use super::*;

    #[test]
    fn test_next_token() {
        let input = String::from(
            r#"
            John 1:3
            1 John 1:3
            1 John 1:3-1
            1 John 1:1,3
            1 John 1:1-3,5
            "#,
        );

        let expected = vec![
            // 1
            Token {
                value: "John".to_string(),
                t_type: TokenEnum::IDENTIFIER,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ":".to_string(),
                t_type: TokenEnum::COLON,
            },
            Token {
                value: "3".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            // 2
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "John".to_string(),
                t_type: TokenEnum::IDENTIFIER,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ":".to_string(),
                t_type: TokenEnum::COLON,
            },
            Token {
                value: "3".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            // 3
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "John".to_string(),
                t_type: TokenEnum::IDENTIFIER,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ":".to_string(),
                t_type: TokenEnum::COLON,
            },
            Token {
                value: "3".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "-".to_string(),
                t_type: TokenEnum::HYPHEN,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            // 4
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "John".to_string(),
                t_type: TokenEnum::IDENTIFIER,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ":".to_string(),
                t_type: TokenEnum::COLON,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ",".to_string(),
                t_type: TokenEnum::COMMA,
            },
            Token {
                value: "3".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            // 5
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "John".to_string(),
                t_type: TokenEnum::IDENTIFIER,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ":".to_string(),
                t_type: TokenEnum::COLON,
            },
            Token {
                value: "1".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: "-".to_string(),
                t_type: TokenEnum::HYPHEN,
            },
            Token {
                value: "3".to_string(),
                t_type: TokenEnum::NUMBER,
            },
            Token {
                value: ",".to_string(),
                t_type: TokenEnum::COMMA,
            },
            Token {
                value: "5".to_string(),
                t_type: TokenEnum::NUMBER,
            },
        ];

        let mut lexer = Tokenizer::new(input);

        for exp in expected {
            // next token
            let token = lexer.next_token();

            // check toke
            assert_eq!(
                exp.t_type, token.t_type,
                "token_type error: expected {:?}, but found {:?}",
                exp.t_type, token.t_type
            );

            // check value
            assert_eq!(
                exp.value, token.value,
                "token_value error: expected {:?}, but found {:?}",
                exp.value, token.value
            );
        }
    }
}
//
