use std::collections::HashMap;

use super::tokenizer::{Token, TokenEnum, Tokenizer};

#[derive(Debug)]
pub struct BibleReference {
    pub book: String,
    pub chapter: u32,
    pub verses: Vec<u32>,
}

struct PageReference {
    chapter: u32,
    verses: Vec<u32>,
}

pub struct AstReference {
    book: Identifier,
    reference: ReferenceLiteral,
}

impl AstReference {
    pub fn inspect(&self) -> String {
        format!("{} {}", self.book.inspect(), self.reference.inspect())
    }
    pub fn eval(&self) -> BibleReference {
        let page = self.reference.eval();
        BibleReference {
            book: self.book.eval(),
            chapter: page.chapter,
            verses: page.verses,
        }
    }
    fn from(list: Vec<AstExpression>) -> Option<Self> {
        if list.len() != 2 {
            return None;
        }

        let ident_ex = match list.first() {
            Some(id) => match id.get::<Identifier>() {
                Some(id) => id.clone(),
                None => return None,
            },
            None => return None,
        };

        let ref_ex = match list.get(1) {
            Some(refe) => match refe.get::<ReferenceLiteral>() {
                Some(refe) => refe.clone(),
                None => return None,
            },

            None => return None,
        };

        Some(AstReference {
            reference: ref_ex,
            book: ident_ex,
        })
    }
}

trait Expression {
    fn expression_node(&self) {}
    fn inspect(&self) -> String;
}

#[derive(Debug, PartialEq, Clone)]
struct Identifier {
    value: String,
    token: Token,
    prefix: Option<NumberLiteral>,
}

impl Identifier {
    fn eval(&self) -> String {
        match &self.prefix {
            Some(pre) => format!("{} {}", pre.inspect(), self.value),
            None => self.value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct NumberLiteral {
    value: u32,
    token: Token,
}

impl NumberLiteral {
    fn new(value: u32) -> Self {
        NumberLiteral {
            value,
            token: Token {
                value: value.to_string(),
                t_type: TokenEnum::NUMBER,
            },
        }
    }

    fn eval(&self) -> u32 {
        self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
struct RangeLiteral {
    end: NumberLiteral,
    start: NumberLiteral,
    token: Token,
}

#[derive(Debug, PartialEq, Clone)]
/// More like chapter and verse
struct ReferenceLiteral {
    chapter: NumberLiteral,
    token: Token,
    verses: Vec<NumberLiteral>,
}

impl ReferenceLiteral {
    fn eval(&self) -> PageReference {
        let chapter = self.chapter.eval();
        let verses = self.verses.iter().map(|a| a.eval()).collect::<Vec<u32>>();

        PageReference { verses, chapter }
    }
}

impl Expression for ReferenceLiteral {
    fn expression_node(&self) {}
    fn inspect(&self) -> String {
        let chapter = self.chapter.inspect();

        let verses = self
            .verses
            .iter()
            .map(|v| v.inspect())
            .collect::<Vec<String>>()
            .join(", ");

        format!("{}:{}", chapter, verses)
    }
}

impl Expression for RangeLiteral {
    fn expression_node(&self) {}
    fn inspect(&self) -> String {
        format!("{}-{}", self.start.inspect(), self.end.inspect())
    }
}

impl Expression for NumberLiteral {
    fn expression_node(&self) {}
    fn inspect(&self) -> String {
        self.token.inspect()
    }
}

impl Expression for Identifier {
    fn expression_node(&self) {}
    fn inspect(&self) -> String {
        self.token.inspect()
    }
}

#[derive(Debug)]
pub struct AstExpression {
    node: Box<dyn std::any::Any>,
    t_type: TokenEnum,
}

impl AstExpression {
    fn new<T>(main_node: T, t_type: TokenEnum) -> AstExpression
    where
        T: Expression + std::any::Any,
    {
        AstExpression {
            node: Box::new(main_node),
            t_type,
        }
    }

    fn get<T: Expression + std::any::Any>(&self) -> Option<&T> {
        self.node.downcast_ref::<T>()
    }
}

type PrefixFn = fn(p: &mut Parser) -> Option<AstExpression>;
type InfixFn = fn(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression>;
type PostfixFn = fn(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression>;

pub struct Parser {
    current_token: Token,
    peek_token: Token,

    tokenizer: Tokenizer,
    // errors: Vec<>
    prefix_fn: HashMap<TokenEnum, PrefixFn>,
    infix_fn: HashMap<TokenEnum, InfixFn>,
    postfix_fn: HashMap<TokenEnum, PostfixFn>,
}

impl Parser {
    pub fn from(input: String) -> Self {
        let tokenizer = Tokenizer::new(input);
        

        Parser::new(tokenizer)
    }
    pub fn parser(input: String) -> Option<AstReference> {
        let mut parser = Parser::from(input);

        AstReference::from(parser.parse_program())
    }

    fn new(mut tokenizer: Tokenizer) -> Self {
        let current_token = tokenizer.next_token();
        let next_token = tokenizer.next_token();

        let mut parser = Parser {
            current_token,
            peek_token: next_token,
            tokenizer,

            prefix_fn: HashMap::new(),
            infix_fn: HashMap::new(),
            postfix_fn: HashMap::new(),
        };

        // prefix
        parser.register_prefix(TokenEnum::IDENTIFIER, parse_prefix_identifier);
        parser.register_prefix(TokenEnum::NUMBER, parse_number);
        // parser.register_prefix(TokenEnum::COLON, parse_pre_verse);

        // infix
        parser.register_infix(TokenEnum::HYPHEN, parse_range);
        // parser.register_infix(TokenEnum::COLON, parse_page);

        // postfix
        parser.register_postfix(TokenEnum::IDENTIFIER, parse_post_identifier);
        parser.register_postfix(TokenEnum::COLON, parse_post_chapter);

        parser
    }

    fn register_prefix(&mut self, t_type: TokenEnum, func: PrefixFn) {
        self.prefix_fn.insert(t_type, func);
    }

    fn register_infix(&mut self, t_type: TokenEnum, func: InfixFn) {
        self.infix_fn.insert(t_type, func);
    }

    fn register_postfix(&mut self, t_type: TokenEnum, func: PostfixFn) {
        self.postfix_fn.insert(t_type, func);
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.tokenizer.next_token();
    }

    fn expect_peek_token(&mut self, token: TokenEnum) -> bool {
        if self.peek_token.t_type == token {
            self.next_token();
            true
        } else {
            // TODO: append to errorr list
            false
        }
    }

    fn parse_program(&mut self) -> Vec<AstExpression> {
        let mut stmts = Vec::new();

        while self.current_token.t_type.ne(&TokenEnum::EOF) {
            // parse statement
            let stmt = self.parse_expression_statement();

            if let Some(stmt) = stmt { stmts.push(stmt) };

            self.next_token();
        }

        stmts
    }

    fn parse_expression_statement(&mut self) -> Option<AstExpression> {
        let expression = self.parse_expression();
        if self.peek_token.t_type == TokenEnum::EOF {
            self.next_token();
        }

        expression
    }

    fn parse_expression(&mut self) -> Option<AstExpression> {
        // identifier
        // chapter
        // :
        // verses expressions

        //get prefix
        let prefix_fn = self.prefix_fn.get(&self.current_token.t_type)?;

        let mut left_exp = prefix_fn(self)?;

        // TODO: run infix check
        while self.current_token.t_type != TokenEnum::COMMA {
            // check for infix
            let infix = match self.infix_fn.get(&self.peek_token.t_type) {
                Some(t) => t,
                None => break,
            };

            // self.next_token();
            self.current_token = self.peek_token.clone();
            self.peek_token = self.tokenizer.next_token();
            if let Some(expression) = infix(self, &left_exp) {
                left_exp = expression
            };
        }

        // TODO: run postfix check

        //get postfix
        if let Some(post_fn) = self.postfix_fn.get(&self.peek_token.t_type) {
            self.current_token = self.peek_token.clone();
            self.peek_token = self.tokenizer.next_token();
            if let Some(post) = post_fn(self, &left_exp) {
                //
                left_exp = post
            };
        };

        Some(left_exp)
    }
}

fn parse_number(p: &mut Parser) -> Option<AstExpression> {
    if p.current_token.t_type != TokenEnum::NUMBER {
        return None;
    }

    let value = match p.current_token.value.clone().parse::<u32>() {
        Ok(val) => val,
        Err(e) => {
            println!("ERROR: could not parse u32 \n{:?}", e);
            return None;
        }
    };

    let number = NumberLiteral {
        token: p.current_token.clone(),
        value,
    };

    Some(AstExpression::new(number, TokenEnum::NUMBER))
}

fn parse_range(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression> {
    if p.current_token.t_type != TokenEnum::HYPHEN {
        return None;
    }

    let lhs_literal = lhs.get::<NumberLiteral>()?;
    let token = p.current_token.clone();

    p.next_token(); // move to rhs

    let rhs = p.parse_expression()?;
    let rhs_literal = rhs.get::<NumberLiteral>()?;

    let range_val = RangeLiteral {
        token,
        start: lhs_literal.clone(),
        end: rhs_literal.clone(),
    };

    Some(AstExpression::new(range_val, TokenEnum::HYPHEN))
}

fn parse_post_identifier(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression> {
    let number_literal = lhs.get::<NumberLiteral>()?;

    let ident_ast_expression = parse_prefix_identifier(p)?;

    let mut ident = match ident_ast_expression.get::<Identifier>() {
        Some(id) => id.clone(),
        None => return None,
    };

    ident.prefix = Some(number_literal.clone());

    Some(AstExpression::new(ident, TokenEnum::IDENTIFIER))
}

fn parse_post_chapter(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression> {
    let number_literal = lhs.get::<NumberLiteral>()?;

    if p.current_token.t_type != TokenEnum::COLON {
        return None;
    }

    let verses = parse_verse(p)?;

    let chapter = ReferenceLiteral {
        token: Token {
            t_type: TokenEnum::COLON,
            value: p.current_token.value.clone(),
        },
        chapter: number_literal.clone(),
        verses,
    };

    Some(AstExpression::new(chapter, TokenEnum::COLON))
}

fn parse_verse(p: &mut Parser) -> Option<Vec<NumberLiteral>> {
    if p.current_token.t_type != TokenEnum::COLON {
        return None;
    }

    p.next_token();

    let mut verses = Vec::new();

    while p.current_token.t_type.ne(&TokenEnum::EOF) {
        let verse = p.parse_expression();

        if let Some(v) = verse {
            //
            // verses.push(v);

            if let Some(ident) = v.get::<NumberLiteral>() {
                let id = ident.clone();
                verses.push(id);
            } else if let Some(range) = v.get::<RangeLiteral>() {
                let range_literal = range.clone();
                let min = u32::min(range_literal.start.value, range_literal.end.value);
                let max = u32::max(range_literal.start.value, range_literal.end.value);
                let gap = min..=max;

                for value in gap {
                    verses.push(NumberLiteral::new(value));
                }
            }
        };

        p.next_token();
    }

    Some(verses)
}

fn parse_prefix_identifier(p: &mut Parser) -> Option<AstExpression> {
    if p.current_token.t_type != TokenEnum::IDENTIFIER {
        return None;
    }

    let ident = Identifier {
        token: p.current_token.clone(),
        value: p.current_token.value.clone(),
        prefix: None,
    };

    Some(AstExpression::new(ident, TokenEnum::IDENTIFIER))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_identifier() {
        let inputs = [String::from("John"),
            String::from("3 John")];
        let expected = [AstExpression::new(
                Identifier {
                    prefix: None,
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                TokenEnum::IDENTIFIER,
            ),
            AstExpression::new(
                Identifier {
                    prefix: Some(NumberLiteral {
                        token: Token {
                            t_type: TokenEnum::NUMBER,
                            value: String::from("3"),
                        },
                        value: 3,
                    }),
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                TokenEnum::IDENTIFIER,
            )];

        for (i, input) in inputs.iter().enumerate() {
            let tokenizer = Tokenizer::new(input.to_string());
            let mut parser = Parser::new(tokenizer);
            let stmts = parser.parse_program();

            let actual = stmts.first().unwrap();
            _test_identifier(i as u32, expected.get(i).unwrap(), actual);
        }
    }

    fn _test_identifier(index: u32, expected: &AstExpression, actual: &AstExpression) {
        let expected_val = expected.get::<Identifier>().unwrap();
        let actual_exp = actual.get::<Identifier>();

        // ==
        assert!(
            actual_exp.is_some(),
            "TEST {index}: Expected [Identity] got [{:?}]",
            actual_exp
        );

        let actual_val = actual_exp.unwrap();

        assert_eq!(
            actual_val.value, expected_val.value,
            "expected [{}], got [{}]",
            expected_val.value, actual_val.value
        );

        assert_eq!(
            actual_val.token.t_type, expected_val.token.t_type,
            "expected [{:?}], got [{:?}]",
            expected_val.token.t_type, actual_val.token.t_type
        );

        assert_eq!(
            actual_val, expected_val,
            "expected [{:?}], got [{:?}]",
            expected_val, actual_val
        );
    }

    #[test]
    fn test_number_literal() {
        let input = String::from("1");
        let expected = AstExpression::new(
            NumberLiteral {
                value: 1,
                token: Token {
                    t_type: TokenEnum::NUMBER,
                    value: String::from("1"),
                },
            },
            TokenEnum::NUMBER,
        );

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        let expression = stmts.first().unwrap();
        let ident = expression.get::<NumberLiteral>();
        let tt = expected.get::<NumberLiteral>().unwrap();

        // ==
        assert!(ident.is_some(), "Expected NumberLiteral");

        let ident = ident.unwrap();

        assert_eq!(
            ident.value, tt.value,
            "expected [{}], got [{}]",
            tt.value, ident.value
        );

        assert_eq!(
            ident.token.t_type, tt.token.t_type,
            "expected [{:?}], got [{:?}]",
            tt.token.t_type, ident.token.t_type
        );
    }

    #[test]
    fn test_range_literal() {
        let input = String::from("1-4");
        let expected = AstExpression::new(
            RangeLiteral {
                start: NumberLiteral {
                    value: 1,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("1"),
                    },
                },
                end: NumberLiteral {
                    value: 4,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("4"),
                    },
                },
                token: Token {
                    t_type: TokenEnum::HYPHEN,
                    value: String::from("-"),
                },
            },
            TokenEnum::HYPHEN,
        );

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        let expression = stmts.first().unwrap();
        let ident = expression.get::<RangeLiteral>();
        let tt = expected.get::<RangeLiteral>().unwrap();

        // ==
        assert!(ident.is_some(), "Expected RangeLiteral");

        let ident = ident.unwrap();

        assert_eq!(
            tt.start, ident.start,
            "expected [{:?}], got [{:?}]",
            tt.start, ident.start,
        );

        assert_eq!(
            ident.token.t_type, tt.token.t_type,
            "expected [{:?}], got [{:?}]",
            tt.token.t_type, ident.token.t_type
        );
    }

    #[test]
    fn test_reference_literal() {
        let input = String::from("3:3");
        let expected = AstExpression::new(
            ReferenceLiteral {
                token: Token {
                    t_type: TokenEnum::COLON,
                    value: String::from(":"),
                },
                chapter: NumberLiteral {
                    value: 3,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("3"),
                    },
                },
                verses: vec![NumberLiteral::new(3)],
            },
            TokenEnum::NUMBER,
        );

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        let expression = stmts.first().unwrap();

        _test_reference_literal(&expected, expression);
    }

    fn _test_reference_literal(expected: &AstExpression, actual: &AstExpression) {
        let ident = actual.get::<ReferenceLiteral>();
        let tt = expected.get::<ReferenceLiteral>().unwrap();

        // ==
        assert!(ident.is_some(), "Expected ChapterLiteral");

        let ident = ident.unwrap();

        assert_eq!(
            ident.chapter, tt.chapter,
            "expected [{:?}], got [{:?}]",
            tt.chapter, ident.chapter
        );

        tt.verses.iter().zip(ident.verses.clone()).all(|(e, a)| {
            assert_eq!(e.clone(), a.clone(), "expected [{:?}], got [{:?}]", e, a,);
            e.value == a.value
        });

        assert_eq!(
            ident.token.t_type, tt.token.t_type,
            "expected [{:?}], got [{:?}]",
            tt.token.t_type, ident.token.t_type
        );
    }

    #[test]
    fn test_list_operator() {
        let input = String::from("1,4,3");
        let expected_list = [AstExpression::new(
                NumberLiteral {
                    value: 1,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("1"),
                    },
                },
                TokenEnum::NUMBER,
            ),
            AstExpression::new(
                NumberLiteral {
                    value: 4,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("4"),
                    },
                },
                TokenEnum::NUMBER,
            ),
            AstExpression::new(
                NumberLiteral {
                    value: 3,
                    token: Token {
                        t_type: TokenEnum::NUMBER,
                        value: String::from("3"),
                    },
                },
                TokenEnum::NUMBER,
            )];

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        for (i, expected) in expected_list.iter().enumerate() {
            let stmt = stmts.get(i);
            assert!(stmt.is_some(), "Expected RangeLiteral");

            let actual = stmt.unwrap().get::<NumberLiteral>();
            let expected_value = expected.get::<NumberLiteral>().unwrap();

            // ==
            assert!(actual.is_some(), "Expected RangeLiteral");

            let actual = actual.unwrap();

            assert_eq!(
                expected_value, actual,
                "expected [{:?}], got [{:?}]",
                expected_value, actual,
            );

            assert_eq!(
                actual.token.t_type, expected_value.token.t_type,
                "expected [{:?}], got [{:?}]",
                expected_value.token.t_type, actual.token.t_type
            );
        }
    }

    #[test]
    fn test_program() {
        let input = String::from("1 John 1:1-3,5");
        let expected_list = [AstExpression::new(
                Identifier {
                    prefix: Some(NumberLiteral::new(1)),
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                TokenEnum::IDENTIFIER,
            ),
            AstExpression::new(
                ReferenceLiteral {
                    token: Token {
                        t_type: TokenEnum::COLON,
                        value: String::from(":"),
                    },
                    chapter: NumberLiteral::new(1),
                    verses: vec![
                        NumberLiteral::new(1),
                        NumberLiteral::new(2),
                        NumberLiteral::new(3),
                        NumberLiteral::new(5),
                    ],
                },
                TokenEnum::COLON,
            )];

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        {
            let stmt = stmts.first(); // identifier
            _test_identifier(0, expected_list.first().unwrap(), stmt.unwrap());
        }

        {
            let stmt = stmts.get(1); // identifier
            _test_reference_literal(expected_list.get(1).unwrap(), stmt.unwrap());
        }

        // for (i, expected) in expected_list.iter().enumerate() {
        //     let stmt = stmts.get(i);
        //     assert!(stmt.is_some(), "Expected [Some] got [None]");
        //
        //     let actual = stmt.unwrap().get::<NumberLiteral>();
        //     let expected_value = expected.get::<NumberLiteral>().unwrap();
        //
        //     // ==
        //     assert!(actual.is_some(), "Expected [{}]",);
        //
        //     let actual = actual.unwrap();
        //
        //     assert_eq!(
        //         expected_value, actual,
        //         "expected [{:?}], got [{:?}]",
        //         expected_value, actual,
        //     );
        //
        //     assert_eq!(
        //         actual.token.t_type, expected_value.token.t_type,
        //         "expected [{:?}], got [{:?}]",
        //         expected_value.token.t_type, actual.token.t_type
        //     );
        // }
    }
}
