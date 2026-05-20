use std::collections::HashMap;

use super::tokenizer::{Token, TokenEnum, Tokenizer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleReference {
    pub book: String,
    pub chapter: u32,
    pub verses: Vec<u32>,
}

struct PageReference {
    chapter: u32,
    verses: Vec<u32>,
}

trait Expression {
    fn inspect(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct PassageLiteral {
    book: Identifier,
    reference: ReferenceLiteral,
}

impl PassageLiteral {
    pub fn eval(&self) -> BibleReference {
        let page = self.reference.eval();
        BibleReference {
            book: self.book.eval(),
            chapter: page.chapter,
            verses: page.verses,
        }
    }
}

impl Expression for PassageLiteral {
    fn inspect(&self) -> String {
        let book = self.book.inspect();
        let chapter = self.reference.chapter.inspect();

        let verses = self
            .reference
            .verses
            .iter()
            .map(|v| v.inspect())
            .collect::<Vec<String>>()
            .join(", ");

        format!("{book} {chapter}:{verses}")
    }
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
    fn inspect(&self) -> String {
        format!("{}-{}", self.start.inspect(), self.end.inspect())
    }
}

impl Expression for NumberLiteral {
    fn inspect(&self) -> String {
        self.token.inspect()
    }
}

impl Expression for Identifier {
    fn inspect(&self) -> String {
        self.token.inspect()
    }
}

impl FromAstNode for Identifier {
    fn from_node(node: &AstNode) -> Option<&Self> {
        match node {
            AstNode::Identifier(val) => Some(val),
            _ => return None,
        }
    }
}

impl FromAstNode for NumberLiteral {
    fn from_node(node: &AstNode) -> Option<&Self> {
        match node {
            AstNode::NumberLiteral(val) => Some(val),
            _ => return None,
        }
    }
}

impl FromAstNode for RangeLiteral {
    fn from_node(node: &AstNode) -> Option<&Self> {
        match node {
            AstNode::RangeLiteral(val) => Some(val),
            _ => return None,
        }
    }
}
impl FromAstNode for ReferenceLiteral {
    fn from_node(node: &AstNode) -> Option<&Self> {
        match node {
            AstNode::ReferenceLiteral(val) => Some(val),
            _ => return None,
        }
    }
}

impl FromAstNode for PassageLiteral {
    fn from_node(node: &AstNode) -> Option<&Self> {
        match node {
            AstNode::AstReferenceLiteral(val) => Some(val),
            _ => return None,
        }
    }
}

#[derive(Debug, Clone)]
enum AstNode {
    Identifier(Identifier),
    NumberLiteral(NumberLiteral),
    RangeLiteral(RangeLiteral),
    ReferenceLiteral(ReferenceLiteral),
    AstReferenceLiteral(PassageLiteral),
}

trait FromAstNode {
    fn from_node(node: &AstNode) -> Option<&Self>;
}

impl AstNode {
    fn from_node<T: FromAstNode>(&self) -> Option<&T> {
        T::from_node(self)
    }
}

#[derive(Debug, Clone)]
pub struct AstExpression {
    node: AstNode,
}

impl AstExpression {
    fn new(main_node: AstNode) -> AstExpression {
        AstExpression { node: main_node }
    }

    fn get<T: FromAstNode>(&self) -> Option<&T> {
        self.node.from_node::<T>()
    }
}

type PrefixFn = fn(p: &mut Parser) -> Option<AstExpression>;
type InfixFn = fn(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression>;
type PostfixFn = fn(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression>;

pub struct Parser {
    current_token: Token,
    peek_token: Token,
    prev_token: Token,

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

    pub fn parse(input: String) -> Vec<BibleReference> {
        let mut parser = Parser::from(input);

        parser
            .parse_program()
            .iter()
            .filter_map(|v| v.get::<PassageLiteral>())
            .map(|v| v.eval())
            .collect()
    }

    fn new(mut tokenizer: Tokenizer) -> Self {
        let current_token = tokenizer.next_token();
        let next_token = tokenizer.next_token();

        let mut parser = Parser {
            current_token,
            peek_token: next_token,
            prev_token: Token {
                t_type: TokenEnum::EOF,
                value: String::from('\0'),
            },

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
        self.prev_token = self.current_token.clone();
        self.current_token = self.peek_token.clone();
        self.peek_token = self.tokenizer.next_token();
    }

    fn _expect_peek_token(&mut self, token: TokenEnum) -> bool {
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

            if let Some(stmt) = stmt {
                stmts.push(stmt)
            } else {
            };

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

        while self.current_token.t_type.ne(&TokenEnum::COMMA)
            && self.current_token.t_type.ne(&TokenEnum::SEMICOLON)
        {
            // check for infix
            let t_type = self.peek_token.t_type.clone();
            if !self.infix_fn.contains_key(&t_type) {
                break;
            }

            self.next_token();

            let infix = match self.infix_fn.get(&t_type) {
                Some(t) => t,
                None => break,
            };
            if let Some(expression) = infix(self, &left_exp) {
                left_exp = expression
            };
        }

        //get postfix
        let peek_ttype = self.peek_token.t_type.clone();
        if let Some(post_fn) = self.postfix_fn.get(&peek_ttype)
            && let Some(post) = post_fn(self, &left_exp)
        {
            left_exp = post
        };

        let val = match left_exp.node {
            AstNode::Identifier(ident) => {
                self.next_token(); // move from identifier to number
                let chapter = parse_number(self)?;
                let chapter_ref = parse_post_chapter(self, &chapter)?
                    .node
                    .from_node::<ReferenceLiteral>()?
                    .clone();
                let book_ref = PassageLiteral {
                    book: ident,
                    reference: chapter_ref,
                };
                AstExpression::new(AstNode::AstReferenceLiteral(book_ref))
            }
            _ => left_exp,
        };

        Some(val)
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

    Some(AstExpression::new(AstNode::NumberLiteral(number)))
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

    Some(AstExpression::new(AstNode::RangeLiteral(range_val)))
}

fn parse_post_identifier(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression> {
    let number_literal = lhs.get::<NumberLiteral>()?;
    p.next_token();

    let ident_ast_expression = parse_prefix_identifier(p)?;

    let mut ident = match ident_ast_expression.get::<Identifier>() {
        Some(id) => id.clone(),
        None => return None,
    };

    ident.prefix = Some(number_literal.clone());

    Some(AstExpression::new(AstNode::Identifier(ident)))
}

fn parse_post_chapter(p: &mut Parser, lhs: &AstExpression) -> Option<AstExpression> {
    let number_literal = lhs.get::<NumberLiteral>()?;

    if p.peek_token.t_type != TokenEnum::COLON {
        return None;
    }

    p.next_token();

    let token = p.current_token.clone();
    let verses = parse_verse(p)?;

    let chapter = ReferenceLiteral {
        token,
        chapter: number_literal.clone(),
        verses,
    };

    Some(AstExpression::new(AstNode::ReferenceLiteral(chapter)))
}

fn parse_verse(p: &mut Parser) -> Option<Vec<NumberLiteral>> {
    if p.current_token.t_type != TokenEnum::COLON {
        return None;
    }

    p.next_token();

    let mut verses = Vec::new();

    let mut hold = None;

    if let Some(num) = parse_number(p)
        && let Some(num) = num.get::<NumberLiteral>()
    {
        hold = Some(num.clone());
    };

    while p.current_token.t_type.ne(&TokenEnum::EOF)
        && p.current_token.t_type.ne(&TokenEnum::SEMICOLON)
    {
        if p.peek_token.t_type == TokenEnum::IDENTIFIER {
            break;
        }
        let verse = p.parse_expression();

        if let Some(v) = verse {
            if let Some(ident) = v.get::<NumberLiteral>() {
                let id = ident.clone();
                verses.push(id);
            } else if let Some(range) = v.get::<RangeLiteral>() {
                let range_literal = range.clone();
                let min = u32::min(range_literal.start.value, range_literal.end.value);
                let max = u32::max(range_literal.start.value, range_literal.end.value);

                for value in min..=max {
                    verses.push(NumberLiteral::new(value));
                }
            } else {
                break;
            }
        }

        p.next_token();
    }

    if verses.is_empty()
        && let Some(num) = hold
    {
        verses.push(num);
    }

    Some(verses)
}

fn parse_prefix_identifier(p: &mut Parser) -> Option<AstExpression> {
    if p.current_token.t_type != TokenEnum::IDENTIFIER {
        return None;
    }

    if p.peek_token.t_type != TokenEnum::NUMBER {
        return None;
    }

    let ident = Identifier {
        token: p.current_token.clone(),
        value: p.current_token.value.clone(),
        prefix: None,
    };

    Some(AstExpression::new(AstNode::Identifier(ident)))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_identifier() {
        let inputs = [String::from("John 1:1")];
        let expected = [AstExpression::new(AstNode::AstReferenceLiteral(
            PassageLiteral {
                book: Identifier {
                    prefix: None,
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                reference: ReferenceLiteral {
                    chapter: NumberLiteral::new(1),
                    token: Token {
                        t_type: TokenEnum::COLON,
                        value: String::from(":"),
                    },
                    verses: vec![NumberLiteral::new(1)],
                },
            },
        ))];

        for (i, input) in inputs.iter().enumerate() {
            let tokenizer = Tokenizer::new(input.to_string());
            let mut parser = Parser::new(tokenizer);
            let stmts = parser.parse_program();

            let actual = stmts.first().unwrap();
            _test_identifier(i as u32, expected.get(i).unwrap(), actual);
        }
    }

    #[test]
    fn test_prefix_identifier() {
        let inputs = [String::from("3 John 1:1")];
        let expected = [AstExpression::new(AstNode::AstReferenceLiteral(
            PassageLiteral {
                book: Identifier {
                    prefix: Some(NumberLiteral::new(3)),
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                reference: ReferenceLiteral {
                    chapter: NumberLiteral::new(1),
                    token: Token {
                        t_type: TokenEnum::COLON,
                        value: String::from(":"),
                    },
                    verses: vec![NumberLiteral::new(1)],
                },
            },
        ))];

        for (i, input) in inputs.iter().enumerate() {
            let tokenizer = Tokenizer::new(input.to_string());
            let mut parser = Parser::new(tokenizer);
            let stmts = parser.parse_program();

            let actual = stmts.first().unwrap();
            _test_identifier(i as u32, expected.get(i).unwrap(), actual);
        }
    }

    fn _test_identifier(index: u32, expected: &AstExpression, actual: &AstExpression) {
        let expected_val = expected.get::<PassageLiteral>().unwrap().book.clone();
        let actual_exp = actual.get::<PassageLiteral>();

        // ==
        assert!(
            actual_exp.is_some(),
            "TEST {index}: Expected [AstReference] got [{:?}]",
            actual_exp
        );

        let actual_val = actual_exp.unwrap().book.clone();

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
        let expected = AstExpression::new(AstNode::NumberLiteral(NumberLiteral {
            value: 1,
            token: Token {
                t_type: TokenEnum::NUMBER,
                value: String::from("1"),
            },
        }));

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
        let expected = AstExpression::new(AstNode::RangeLiteral(RangeLiteral {
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
        }));

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
        let expected = AstExpression::new(AstNode::ReferenceLiteral(ReferenceLiteral {
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
        }));

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
        let expected_list = [
            AstExpression::new(AstNode::NumberLiteral(NumberLiteral {
                value: 1,
                token: Token {
                    t_type: TokenEnum::NUMBER,
                    value: String::from("1"),
                },
            })),
            AstExpression::new(AstNode::NumberLiteral(NumberLiteral {
                value: 4,
                token: Token {
                    t_type: TokenEnum::NUMBER,
                    value: String::from("4"),
                },
            })),
            AstExpression::new(AstNode::NumberLiteral(NumberLiteral {
                value: 3,
                token: Token {
                    t_type: TokenEnum::NUMBER,
                    value: String::from("3"),
                },
            })),
        ];

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
        let input = String::from("1 John 1:1-3,5,7-10");
        let expected_list = [AstExpression::new(AstNode::AstReferenceLiteral(
            PassageLiteral {
                book: Identifier {
                    prefix: Some(NumberLiteral::new(1)),
                    value: String::from("John"),
                    token: Token {
                        t_type: TokenEnum::IDENTIFIER,
                        value: String::from("John"),
                    },
                },
                reference: ReferenceLiteral {
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
                        NumberLiteral::new(7),
                        NumberLiteral::new(8),
                        NumberLiteral::new(9),
                    ],
                },
            },
        ))];

        let tokenizer = Tokenizer::new(input);
        let mut parser = Parser::new(tokenizer);
        let stmts = parser.parse_program();

        {
            let stmt = stmts.first(); // identifier
            _test_identifier(0, expected_list.first().unwrap(), stmt.unwrap());
        }

        // {
        //     let stmt = stmts.get(1); // identifier
        //     _test_reference_literal(expected_list.get(1).unwrap(), stmt.unwrap());
        // }

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
