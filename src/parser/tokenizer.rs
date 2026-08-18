use gtk::glib::char;

const BIBLE_BOOK_IDENTIFIERS: [&str; 56] = [
    "genesis",
    "exodus",
    "leviticus",
    "numbers",
    "deuteronomy",
    "joshua",
    "judges",
    "ruth",
    "samuel",
    "kings",
    "chronicles",
    "ezra",
    "nehemiah",
    "esther",
    "job",
    "psalms",
    "proverbs",
    "ecclesiastes",
    "song of solomon",
    "isaiah",
    "jeremiah",
    "lamentations",
    "ezekiel",
    "daniel",
    "hosea",
    "joel",
    "amos",
    "obadiah",
    "jonah",
    "micah",
    "nahum",
    "habakkuk",
    "zephaniah",
    "haggai",
    "zechariah",
    "malachi",
    "matthew",
    "mark",
    "luke",
    "john",
    "acts",
    "romans",
    "corinthians",
    "galatians",
    "ephesians",
    "philippians",
    "colossians",
    "thessalonians",
    "timothy",
    "titus",
    "philemon",
    "hebrews",
    "james",
    "peter",
    "jude",
    "revelation",
];

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TokenEnum {
    Number,
    Identifier,
    Colon,
    Semicolon,
    Chapter,
    Hyphen,
    Comma,
    Illegal,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub t_type: TokenEnum,
    pub value: String,
}

impl Token {
    fn new(t_type: TokenEnum, value: String) -> Self {
        Self { t_type, value }
    }
    pub fn inspect(&self) -> String {
        self.value.clone()
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

        let input: String = String::from("  ") + &inp;

        Tokenizer {
            char: input.chars().nth(0).expect("Input cannot be empty"),
            position: 0,
            peek_position: 1,
            input,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_white_space();
        // println!("CHAR {}", self.char);

        let token = match self.char {
            ':' => Token {
                t_type: TokenEnum::Colon,
                value: String::from(":"),
            },
            ';' => Token {
                t_type: TokenEnum::Semicolon,
                value: String::from(";"),
            },
            '-' => Token {
                t_type: TokenEnum::Hyphen,
                value: String::from("-"),
            },
            ',' => Token {
                t_type: TokenEnum::Comma,
                value: String::from(","),
            },

            // ===
            // handle EOF
            // ===
            '\0' => Token {
                t_type: TokenEnum::Eof,
                value: String::from('\0'),
            },

            ch => {
                if Tokenizer::is_letter(ch) {
                    // read string
                    let text = self.read_string();

                    return match text.to_lowercase().as_str() {
                        "chapter" => Token::new(TokenEnum::Chapter, text),
                        "verse" | "verses" => Token::new(TokenEnum::Colon, text),
                        "to" | "through" => Token::new(TokenEnum::Hyphen, text),
                        "and" => Token::new(TokenEnum::Comma, text),
                        s if BIBLE_BOOK_IDENTIFIERS.contains(&text.to_lowercase().as_str()) => {
                            Token::new(TokenEnum::Identifier, text)
                        }
                        _ => Token::new(TokenEnum::Illegal, text),
                    };
                }

                if Tokenizer::is_digit(ch) {
                    // read digit
                    let num = self.read_digit();
                    return Token {
                        t_type: TokenEnum::Number,
                        value: num,
                    };
                }

                Token {
                    value: String::from(self.char),
                    t_type: TokenEnum::Illegal,
                }
            }
        };

        self.read_char();
        token
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

        self.input
            .get(start as usize..self.position as usize)
            .unwrap() // TODO: handle error None arm
            .to_string()
    }

    fn read_digit(&mut self) -> String {
        let start = self.position;
        while Tokenizer::is_digit(self.char) {
            self.read_char();
        }

        self.input
            .get(start as usize..self.position as usize)
            .unwrap() // TODO: handle error None arm
            .to_string()
    }

    pub fn read_char(&mut self) {
        if self.peek_position as usize >= self.input.len() {
            self.char = '\0'
        } else if let Some(ch) = self.input.chars().nth(self.peek_position as usize) {
            self.char = ch
        }

        self.position = self.peek_position;
        self.peek_position += 1;
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
            1 John 1:1-3,5;
            John chapter 1 verse 3
            to through and
            rubish
            "#,
        );

        let expected = vec![
            // 1
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, ":".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            // 2
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, ":".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            // 3
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, ":".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            Token::new(TokenEnum::Hyphen, "-".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            // 4
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, ":".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Comma, ",".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            // 5
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, ":".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Hyphen, "-".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            Token::new(TokenEnum::Comma, ",".to_string()),
            Token::new(TokenEnum::Number, "5".to_string()),
            Token::new(TokenEnum::Semicolon, ";".to_string()),
            // 6
            Token::new(TokenEnum::Identifier, "John".to_string()),
            Token::new(TokenEnum::Chapter, "chapter".to_string()),
            Token::new(TokenEnum::Number, "1".to_string()),
            Token::new(TokenEnum::Colon, "verse".to_string()),
            Token::new(TokenEnum::Number, "3".to_string()),
            // 7
            Token::new(TokenEnum::Hyphen, "to".to_string()),
            Token::new(TokenEnum::Hyphen, "through".to_string()),
            Token::new(TokenEnum::Comma, "and".to_string()),
            Token::new(TokenEnum::Illegal, "rubish".to_string()),
        ];

        let mut lexer = Tokenizer::new(input);

        for exp in expected {
            // next token
            let token = lexer.next_token();

            // check tokem
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
