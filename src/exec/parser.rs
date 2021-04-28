use crate::exec::{ActualArgumentExpressions, ExecResult, ExecError, Expression, Value};

#[derive(Clone, Copy, Debug)]
pub struct SourceLocation
{
    offset: usize,
}

impl SourceLocation
{
    pub fn inbuilt() -> Self
    {
        SourceLocation{ offset: 0 }
    }

    fn from_offset(offset: usize) -> Self
    {
        SourceLocation{ offset }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokenKind
{
    UnsignedInt,
    Identifier,
    Operator,
    InvalidChar,
    Eof,
}

#[derive(Debug, Clone)]
struct Token<'a>
{
    kind: TokenKind,
    source: SourceLocation,
    text: &'a str,
}

pub fn parse(input: &str) -> ExecResult<Box<Expression>>
{
    let mut parser = Parser::new(input)?;

    let result = parse_expression(&mut parser)?;

    if !parser.peek_kind(TokenKind::Eof)
    {
        return Err(parser.err_expected("<EOF>"));
    }

    Ok(result)
}

fn parse_expression<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    let mut result = parse_term(parser)?;

    while parser.peek_ch('+') || parser.peek_ch('-')
    {
        let op = parser.next();

        let sub = parse_term(parser)?;

        let actual_args = ActualArgumentExpressions::Positional(vec![result, sub]);

        let function = Expression::new_read_named_var(op.source, op.text.to_owned());

        result = Expression::new_call(op.source, function, actual_args);
    }

    Ok(result)
}

fn parse_term<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    let mut result = parse_call(parser)?;

    while parser.peek_ch('*') || parser.peek_ch('/')
    {
        let op = parser.next();

        let sub = parse_call(parser)?;

        let actual_args = ActualArgumentExpressions::Positional(vec![result, sub]);

        let function = Expression::new_read_named_var(op.source, op.text.to_owned());

        result = Expression::new_call(op.source, function, actual_args);
    }

    Ok(result)
}

fn parse_call<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    let mut result = parse_factor(parser)?;

    while parser.peek_ch('(') || parser.peek_ch('{')
    {
        let open = parser.next();

        if open.text == "("
        {
            let mut args = Vec::new();

            loop
            {
                if parser.peek_ch(')')
                {
                    let _ = parser.next();
                    break;
                }

                args.push(parse_expression(parser)?);

                if parser.peek_ch(')')
                {
                    continue;
                }
                else if parser.peek_ch(',')
                {
                    let _ = parser.next();
                    continue;
                }

                return Err(parser.err_expected("',' or ')'"));
            }

            result = Expression::new_call(open.source, result, ActualArgumentExpressions::Positional(args));
        }
        else // '{'
        {
            let mut args = Vec::new();

            loop
            {
                if parser.peek_ch('}')
                {
                    let _ = parser.next();
                    break;
                }

                if !parser.peek_kind(TokenKind::Identifier)
                {
                    return Err(parser.err_expected("Identitfier"));
                }
                let name = parser.next();

                parser.expect_ch(':')?;

                args.push((name.text.to_string(), parse_expression(parser)?));

                if parser.peek_ch('}')
                {
                    continue;
                }
                else if parser.peek_ch(',')
                {
                    let _ = parser.next();
                    continue;
                }

                return Err(parser.err_expected("',' or '}'"));
            }

            result = Expression::new_call(open.source, result, ActualArgumentExpressions::Named(args));
        }
    }

    Ok(result)
}

fn parse_factor<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    if parser.peek_kind(TokenKind::UnsignedInt)
    {
        let token = parser.next();
        return match token.text.parse::<i64>()
        {
            Err(_) => Err(ExecError::new(token.source, "Invalid integer constant")),
            Ok(val) => Ok(Expression::new_constant(Value::new_int(token.source, val)))
        };
    }
    else if parser.peek_kind(TokenKind::Identifier)
    {
        let token = parser.next();

        return Ok(Expression::new_read_named_var(token.source, token.text.to_owned()));
    }
    else if parser.peek_ch('(')
    {
        let _ = parser.next();

        let result = parse_expression(parser)?;

        parser.expect_ch(')')?;

        return Ok(result);
    }
    else
    {
        return Err(parser.err_expected("Unsigned int, identifier or open parenthesis".to_owned()));
    }
}

struct Parser<'a>
{
    tokens: Vec<Token<'a>>,
    index: usize,
}

impl<'a> Parser<'a>
{
    fn new(input: &'a str) -> ExecResult<Self>
    {
        let mut tokens = parse_tokens(input)?;
        tokens.push(Token
        {
            kind: TokenKind::Eof,
            source: SourceLocation::from_offset(input.len()),
            text: "",
        });

        Ok(Parser
        {
            tokens,
            index: 0,
        })
    }

    fn peek_ch(&self, ch: char) -> bool
    {
        (self.tokens[self.index].text.chars().nth(0) == Some(ch))
            && (self.tokens[self.index].text.chars().count() == 1)
    }

    fn peek_kind(&self, kind: TokenKind) -> bool
    {
        self.tokens[self.index].kind == kind
    }

    fn next(&mut self) -> Token<'a>
    {
        let result = self.tokens[self.index].clone();
        self.index += 1;
        return result;
    }

    fn expect_ch(&mut self, ch: char) -> ExecResult<()>
    {
        if !self.peek_ch(ch)
        {
            return Err(self.err_expected(format!("\'{}\'", ch)));
        }
        self.next();
        Ok(())
    }

    fn err_expected<S: Into<String>>(&mut self, msg: S) -> ExecError
    {
        ExecError::new(self.tokens[self.index].source, format!("Expected {}", msg.into()))
    }
}

struct Lexer<'a>
{
    input: &'a str,
    it: std::str::CharIndices<'a>,
    peek: Option<(usize, char)>,
    token_start_offset: usize,
    token_len: usize,
}

impl<'a> Lexer<'a>
{
    fn new(input: &'a str) -> Self
    {
        let mut it = input.char_indices();
        let peek = it.next();

        Lexer
        {
            input,
            it,
            peek,
            token_start_offset: 0,
            token_len: 0,
        }
    }

    fn peek(&mut self) -> char
    {
        if let Some((_, ch)) = self.peek
        {
            return ch;
        }

        return '\0';
    }

    fn finished(&self) -> bool
    {
        self.peek.is_none()
    }

    fn accept_char(&mut self)
    {
        let next_peek = self.it.next();

        match next_peek
        {
            Some((offset, _)) => self.token_len = offset - self.token_start_offset,
            None => self.token_len = self.input.len() - self.token_start_offset,
        };

        self.peek = next_peek;
    }

    fn take_token(&mut self, kind: TokenKind) -> Token<'a>
    {
        let result = Token
        {
            kind: kind,
            source: SourceLocation::from_offset(self.token_start_offset),
            text: &self.input[self.token_start_offset..(self.token_start_offset + self.token_len)],
        };

        match self.peek
        {
            Some((offset, _)) => self.token_start_offset = offset,
            None => self.token_start_offset = self.input.len(),
        }

        result
    }

    fn ignore_token(&mut self)
    {
        let _ = self.take_token(TokenKind::InvalidChar);
    }
}

fn parse_tokens<'a>(input: &'a str) -> ExecResult<Vec<Token<'a>>>
{
    let mut result = Vec::new();
    let mut lexer = Lexer::new(input);

    while !lexer.finished()
    {
        let ch = lexer.peek();

        if (ch >= '0') && (ch <= '9')
        {
            lexer.accept_char();
            while (lexer.peek() >= '0') && (lexer.peek() <= '9')
            {
                lexer.accept_char();
            }
            result.push(lexer.take_token(TokenKind::UnsignedInt));
        }
        else if (ch == '_') || ch.is_alphabetic()
        {
            lexer.accept_char();

            while (lexer.peek() == '|') || lexer.peek().is_alphanumeric()
            {
                lexer.accept_char();
            }
            result.push(lexer.take_token(TokenKind::Identifier));
        }
        else if ch == '+' || ch == '-'
            || ch == '(' || ch == ')'
            || ch == '{' || ch == '}'
            || ch == '*' || ch == '/'
            || ch == '.'
            || ch == ':'
            || ch == ','
        {
            lexer.accept_char();
            result.push(lexer.take_token(TokenKind::Operator));
        }
        else if ch.is_whitespace()
        {
            lexer.accept_char();
            lexer.ignore_token();
        }
        else
        {
            lexer.accept_char();
            result.push(lexer.take_token(TokenKind::InvalidChar));
        }
    }

    Ok(result)
}
