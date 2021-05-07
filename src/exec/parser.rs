use crate::math::Scalar;
use crate::exec::{ActualArgumentExpressions, ExecResult, ExecError, Expression, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation
{
    offset: usize,
}

impl SourceLocation
{
    pub fn inbuilt() -> Self
    {
        SourceLocation{ offset: usize::MAX }
    }

    fn from_offset(offset: usize) -> Self
    {
        SourceLocation{ offset }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokenKind
{
    Scalar,
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

pub fn parse(input: &str) -> ExecResult<Vec<Box<Expression>>>
{
    let mut parser = Parser::new(input)?;

    parse_block_contents(&mut parser)
}

fn parse_block<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    let open = parser.expect_ch('{')?;

    let result = parse_block_contents(parser)?;

    parser.expect_close(open, '}')?;

    Ok(Expression::new_block(result))
}

fn parse_block_contents<'a>(parser: &mut Parser<'a>) -> ExecResult<Vec<Box<Expression>>>
{
    let mut result = Vec::new();

    while !parser.peek_kind(TokenKind::Eof) && !parser.peek_ch('}')
    {
        result.push(parse_expression(parser)?);
    }

    Ok(result)
}

fn parse_expression<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
{
    if parser.peek_keyword("let")
    {
        parser.next();

        let name = parser.expect_kind(TokenKind::Identifier)?;

        parser.expect_ch('=')?;

        let exp = parse_expression(parser)?;

        parser.expect_ch(';')?;

        Ok(Expression::new_write_named_var(name.text.to_owned(), exp))
    }
    else if parser.peek_keyword("function")
    {
        // Parse function keyword

        let func_token = parser.next();

        // Parse optional name

        let mut name_token = None;
        let mut name_string = "<function>".to_owned();

        if parser.peek_kind(TokenKind::Identifier)
        {
            let tok = parser.next();
            name_string = tok.text.to_owned();
            name_token = Some(tok);
        }

        // Parse formal arguments

        let open = parser.expect_ch('(')?;

        let mut formal_arguments = Vec::new();

        loop
        {
            if parser.peek_kind(TokenKind::Identifier)
            {
                formal_arguments.push(parser.next().text.to_owned());

                if parser.peek_ch(',')
                {
                    parser.next();
                    continue;
                }
            }
            parser.expect_close(open, ')')?;
            break;
        }

        // Parse function body

        let exp = parse_block(parser)?;

        // Build resulting expression

        let mut result = Expression::new_function(func_token.source, name_string, formal_arguments, exp);

        if let Some(name_token) = name_token
        {
            result = Expression::new_write_named_var(name_token.text.to_owned(), result);
        }

        Ok(result)
    }
    else if parser.peek_keyword("if")
    {
        let mut conditions = Vec::new();
        let mut alternative = None;

        {
            parser.next();
            let open = parser.expect_ch('(')?;
            let cond = parse_expression(parser)?;
            parser.expect_close(open, ')')?;
            let exp = parse_block(parser)?;

            conditions.push((cond, exp));
        }

        loop
        {
            if parser.peek_keyword("else")
            {
                parser.next();

                if parser.peek_keyword("if")
                {
                    parser.next();
                    let open = parser.expect_ch('(')?;
                    let cond = parse_expression(parser)?;
                    parser.expect_close(open, ')')?;
                    let exp = parse_block(parser)?;
        
                    conditions.push((cond, exp));

                    continue;
                }
                else
                {
                    alternative = Some(parse_block(parser)?);
                    break;
                }
            }
            break;
        }

        Ok(Expression::new_if(conditions, alternative))
    }
    else // general expression
    {
        let mut result = parse_sum(parser)?;

        while parser.peek_keyword("==") || parser.peek_keyword("!=")
        {
            let op = parser.next();

            let sub = parse_sum(parser)?;

            let actual_args = ActualArgumentExpressions::Positional(vec![result, sub]);

            let function = Expression::new_read_named_var(op.source, op.text.to_owned());

            result = Expression::new_call(op.source, function, actual_args);
        }

        Ok(result)
    }
}

fn parse_sum<'a>(parser: &mut Parser<'a>) -> ExecResult<Box<Expression>>
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
    if parser.peek_kind(TokenKind::Scalar)
    {
        let token = parser.next();
        return match token.text.parse::<Scalar>()
        {
            Err(_) => Err(ExecError::new(token.source, "Invalid scalar constant")),
            Ok(val) => Ok(Expression::new_constant(Value::new_scalar(token.source, val)))
        };
    }
    else if parser.peek_kind(TokenKind::Identifier)
    {
        let token = parser.next();

        if token.text == "true"
        {
            return Ok(Expression::new_constant(Value::new_bool(token.source, true)));
        }
        else if token.text == "false"
        {
            return Ok(Expression::new_constant(Value::new_bool(token.source, false)));
        }
        else
        {
            return Ok(Expression::new_read_named_var(token.source, token.text.to_owned()));
        }
    }
    else if parser.peek_ch('<')
    {
        // Vector operator

        let start = parser.next();

        let mut exprs = Vec::new();

        loop
        {
            exprs.push(parse_expression(parser)?);

            if parser.peek_ch(',')
            {
                let _ = parser.next();
                continue;
            }
            else if parser.peek_ch('>')
            {
                let _ = parser.next();

                return Ok(Expression::new_vector(start.source, exprs));
            }
            else
            {
                return Err(parser.err_expected("\",\" or \">\" to end vector"));
            }
        }
    }
    else if parser.peek_ch('-')
    {
        let op = parser.next();

        let arg = parse_factor(parser)?;

        let actual_args = ActualArgumentExpressions::Positional(vec![arg]);

        let function = Expression::new_read_named_var(op.source, "neg".to_owned());

        return Ok(Expression::new_call(op.source, function, actual_args));
    }
    else if parser.peek_ch('(')
    {
        let open = parser.next();

        let result = parse_expression(parser)?;

        parser.expect_close(open, ')')?;

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
        (self.tokens[self.index].kind == TokenKind::Operator)
            && (self.tokens[self.index].text.chars().nth(0) == Some(ch))
            && (self.tokens[self.index].text.chars().count() == 1)
    }

    fn peek_keyword(&self, keyword: &str) -> bool
    {
        self.tokens[self.index].text == keyword
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

    fn expect_ch(&mut self, ch: char) -> ExecResult<Token<'a>>
    {
        if !self.peek_ch(ch)
        {
            return Err(self.err_expected(format!("\'{}\'", ch)));
        }
        Ok(self.next())
    }

    fn expect_close(&mut self, _open: Token<'a>, ch: char) -> ExecResult<Token<'a>>
    {
        self.expect_ch(ch)
    }

    fn expect_kind(&mut self, kind: TokenKind) -> ExecResult<Token<'a>>
    {
        if !self.peek_kind(kind)
        {
            return Err(self.err_expected(format!("{:?}", kind)));
        }
        Ok(self.next())
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

            if lexer.peek() == '.'
            {
                lexer.accept_char();
                while (lexer.peek() >= '0') && (lexer.peek() <= '9')
                {
                    lexer.accept_char();
                }
            }

            result.push(lexer.take_token(TokenKind::Scalar));
        }
        else if (ch == '_') || ch.is_alphabetic()
        {
            lexer.accept_char();

            while (lexer.peek() == '_') || lexer.peek().is_alphanumeric()
            {
                lexer.accept_char();
            }
            result.push(lexer.take_token(TokenKind::Identifier));
        }
        else if ch == '='
        {
            lexer.accept_char();

            if lexer.peek() == '='
            {
                lexer.accept_char();
            }

            result.push(lexer.take_token(TokenKind::Operator));
        }
        else if ch == '!'
        {
            lexer.accept_char();

            if lexer.peek() == '='
            {
                lexer.accept_char();
            }
            
            result.push(lexer.take_token(TokenKind::Operator));
        }
        else if ch == '+' || ch == '-'
            || ch == '*' || ch == '/'
            || ch == '(' || ch == ')'
            || ch == '{' || ch == '}'
            || ch == '<' || ch == '>'
            || ch == '.'
            || ch == ':'
            || ch == ','
            || ch == ';'
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
