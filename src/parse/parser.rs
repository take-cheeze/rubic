use parse::{Tokenizer, Token, Error};
use ast;

use std::iter::Peekable;

/// A parser.
pub struct Parser<I: Iterator<Item=char>>
{
    tokenizer: Peekable<Tokenizer<I>>,
}

/// Something kind-of like a class (i.e. a module).
pub struct ClassLike {
    name: String,
    items: Vec<ast::Item>,
}

impl<I> Parser<I>
    where I: Iterator<Item=char>
{
    /// Creates a new parser.
    pub fn new<A>(input: A) -> Self
        where A: IntoIterator<IntoIter=I, Item=char> {
        Parser { tokenizer: Tokenizer::new(input.into_iter()).peekable() }
    }

    /// Parses the program.
    pub fn parse(mut self) -> Result<ast::Program, Error> {
        let mut items = Vec::new();

        self.eat_whitespace();
        while !self.is_eof() {
            let item = self.parse_item()?;
            items.push(item);

            self.eat_whitespace();
        }

        Ok(ast::Program { items: items })
    }

    fn parse_item(&mut self) -> Result<ast::Item, Error> {
        let item = match &expect::word(self.peek())?[..] {
            "class" => self.parse_class().map(ast::Item::Class),
            "module" => self.parse_module().map(ast::Item::Module),
            "def" => self.parse_function().map(ast::Item::Function),
            _ => self.parse_statement().map(ast::Item::Stmt),
        }?;

        println!("got {:?}", item);
        expect::terminator(self.next())?;

        Ok(item)
    }

    /// Parses a class definition.
    fn parse_class(&mut self) -> Result<ast::Class, Error> {
        self.eat_assert(&Token::class());

        let class_like = self.parse_class_like_thing()?;
        Ok(ast::Class { name: class_like.name, items: class_like.items })
    }

    /// Parses a module definition.
    fn parse_module(&mut self) -> Result<ast::Module, Error> {
        self.eat_assert(&Token::module());

        let class_like = self.parse_class_like_thing()?;
        Ok(ast::Module { name: class_like.name, items: class_like.items })
    }

    /// Parses a function.
    fn parse_function(&mut self) -> Result<ast::Function, Error> {
        self.eat_assert(&Token::def());

        let name = expect::word(self.next())?;
        let _parameters = self.parse_function_parameters()?;
        let mut statements = Vec::new();

        // Expect new line or semicolon after parameters.
        expect::terminator(self.next())?;

        self.until_end(|parser| {
            let statement = parser.parse_statement()?;
            statements.push(statement);
            Ok(())
        })?;

        Ok(ast::Function { name: name, statements: statements })
    }

    fn parse_function_parameters(&mut self) -> Result<(), Error> {
        let peeked_token = expect::something(self.peek())?;

        // Check if there is a parameter list.
        if peeked_token == Token::left_paren() {
            unimplemented!();
        } else {
            // No arguments
            Ok(())
        }
    }

    /// Parses something which looks like a class (and ends with an `end`).
    fn parse_class_like_thing(&mut self) -> Result<ClassLike, Error> {
        let name = expect::word(self.next())?;
        let mut items = Vec::new();

        expect::terminator(self.next())?;

        self.until_end(|parser| {
            let item = parser.parse_item()?;
            items.push(item);
            Ok(())
        })?;

        Ok(ClassLike { name: name, items: items })
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> Result<ast::Stmt, Error> {
        let expr = self.parse_expression()?;
        Ok(ast::Stmt::Expr(expr))
    }

    fn parse_expression(&mut self) -> Result<ast::Expr, Error> {
        match expect::something(self.peek())? {
            Token::Word(..) => {
                let path = self.parse_path()?;
                let arguments = self.parse_arguments()?;

                Ok(ast::CallExpr {
                    callee: path,
                    arguments: arguments,
                }.into())
            },
            Token::String(..) => self.parse_string_expression().map(Into::into),
            token => panic!("don't know how to handle: {:?}", token),
        }
    }

    fn parse_path(&mut self) -> Result<ast::Path, Error> {
        let mut segments = Vec::new();
        let mut last_separator = ast::PathSeparator::Root;

        loop {
            // Read the next word and push it to the list.
            let word = expect::word(self.next())?;
            segments.push(ast::PathSegment {
                separator: last_separator,
                kind: ast::PathSegmentKind::new(word),
            });

            match self.peek() {
                Some(Token::Symbol("::")) => {
                    last_separator = ast::PathSeparator::DoubleColon;
                    self.eat();
                },
                Some(Token::Symbol(".")) => {
                    last_separator = ast::PathSeparator::Dot;
                    self.eat();
                },
                _ => break, // We've finished parsing
            }
        }

        Ok(segments.into_iter().collect())
    }

    fn parse_string_expression(&mut self) -> Result<ast::StringLiteral, Error> {
        let token = self.next().unwrap();

        if let Token::String(s) = token {
            Ok(ast::StringLiteral { value: s })
        } else {
            unreachable!();
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<ast::Argument>, Error> {
        let mut arguments = Vec::new();

        self.until_terminator(|parser| {
            let argument = parser.parse_argument()?;
            arguments.push(argument);
            Ok(())
        })?;

        Ok(arguments)
    }

    fn parse_argument(&mut self) -> Result<ast::Argument, Error> {
        let value = self.parse_expression()?;
        Ok(ast::Argument::Positional(value))
    }

    fn peek(&mut self) -> Option<Token> { self.tokenizer.peek().map(Clone::clone) }
    fn next(&mut self) -> Option<Token> { self.tokenizer.next() }
    fn eat(&mut self) -> Option<Token> { self.next() }

    fn eat_assert(&mut self, token: &Token) {
        let read_token = self.next().expect("no more tokens");
        assert_eq!(read_token, *token, "tokens do not match");
    }

    /// Checks if we've reached the end of file yet.
    fn is_eof(&mut self) -> bool {
        if let Some(token) = self.peek() {
            if let Token::EndOfFile = token { true } else { false }
        } else {
            true
        }
    }

    fn eat_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(Token::EndOfLine) => {
                    self.eat();
                },
                _ => break,
            }
        }
    }

    fn until_end<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnMut(&mut Self) -> Result<(), Error> {
        self.until(|token| *token == Token::end(), f, true)?;
        self.eat_assert(&Token::end());
        Ok(())
    }

    fn until_terminator<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnMut(&mut Self) -> Result<(), Error> {
        self.until(Token::is_terminator, f, false)?;
        self.eat(); // Eat terminator.
        Ok(())
    }

    fn until<P,F>(&mut self,
                  mut pred: P,
                  mut f: F,
                  eat_trailing_whitespace: bool) -> Result<(), Error>
        where P: FnMut(&Token) -> bool, F: FnMut(&mut Self) -> Result<(), Error> {
        // Stop immediately if the predicate is true.
        if self.peek().map(|tok| pred(&tok)).unwrap_or(true) { return Ok(()) };

        self.eat_whitespace();

        while self.peek().map(|tok| !pred(&tok)).unwrap_or(false) {
            f(self)?;

            if eat_trailing_whitespace { self.eat_whitespace(); }
        }

        Ok(())
    }
}

/// Utilities for reading tokens.
mod expect
{
    use parse::{Token, Error, ErrorKind};

    pub fn something(token: Option<Token>) -> Result<Token, Error> {
        if let Some(token) = token {
            Ok(token)
        } else {
            panic!("expected something but got nothing");
        }
    }

    pub fn word(token: Option<Token>) -> Result<String, Error> {
        let token = self::something(token)?;

        if let Token::Word(word) = token {
            Ok(word)
        } else {
            Err(ErrorKind::UnexpectedToken(token, vec![Token::Word("word".to_owned())]).into())
        }
    }


    /// A terminating 'new line' or semicolon.
    pub fn terminator(token: Option<Token>) -> Result<(), Error> {
        match token {
            None |
            Some(Token::EndOfLine) |
                Some(Token::Symbol(";")) => Ok(()),
            Some(token) => Err(ErrorKind::UnexpectedToken(token, vec![Token::Word("terminator".to_owned())]).into()),

        }
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use ast;

    fn parse(s: &str) -> ast::Program {
        Parser::new(s.chars()).parse().expect("failed to parse")
    }

    #[test]
    fn can_parse_single_empty_class() {
        assert_eq!(parse("class Abc\nend"), ast::Program {
            items: vec![ast::Class::new("Abc").into()]
        });
    }

    #[test]
    fn can_parse_multiple_empty_classes() {
        assert_eq!(parse("class Abc\nend\nclass Def\nend"), ast::Program {
            items: vec![ast::Class::new("Abc").into(), ast::Class::new("Def").into()],
        });
    }

    #[test]
    fn can_parse_classes_with_semicolons() {
        assert_eq!(parse("class Abc;end"), ast::Program {
            items: vec![ast::Class::new("Abc").into()],
        });
    }

    #[test]
    fn can_parse_nested_classes() {
        assert_eq!(parse("class Abc; class Def; end; end"), ast::Program {
            items: vec![ast::Class {
                name: "Abc".to_owned(),
                items: vec![ast::Class::new("Def").into()],
            }.into()]
        });
    }

    #[test]
    fn can_parse_single_empty_module() {
        assert_eq!(parse("module Abc\nend"), ast::Program {
            items: vec![ast::Module::new("Abc").into()]
        });
    }

    #[test]
    fn can_parse_nested_modules() {
        assert_eq!(parse("module Abc; module Def; end; end"), ast::Program {
            items: vec![ast::Module {
                name: "Abc".to_owned(),
                items: vec![ast::Module::new("Def").into()],
            }.into()]
        });
    }

    #[test]
    fn can_parse_simple_function() {
        assert_eq!(parse("def abc; end"), ast::Program {
            items: vec![ast::Function {
                name: "abc".to_owned(),
                statements: vec![],
            }.into()]
        });
    }

    #[test]
    fn can_parse_path() {
        assert_eq!(parse("Abc::def.obt"), ast::Program {
            items: vec![ast::Stmt::Expr(ast::CallExpr {
                callee: ast::Path {
                    parts: vec![
                        ast::PathSegment {
                            kind: ast::PathSegmentKind::Constant(ast::Constant("Abc".to_owned())),
                            separator: ast::PathSeparator::Root,
                        },
                        ast::PathSegment {
                            kind: ast::PathSegmentKind::Identifier(ast::Identifier("def".to_owned())),
                            separator: ast::PathSeparator::DoubleColon,
                        },
                        ast::PathSegment {
                            kind: ast::PathSegmentKind::Identifier(ast::Identifier("obt".to_owned())),
                            separator: ast::PathSeparator::Dot,
                        },
                    ],
                },
                arguments: Vec::new(),
            }.into()).into()]
        });
    }
}
