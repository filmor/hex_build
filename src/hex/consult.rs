use std::iter::Peekable;

use erl_tokenize::{values::Symbol, Lexer, LexicalToken};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid token type")]
    InvalidToken(LexicalToken),
    #[error("end of input")]
    EndOfInput,
    #[error("tokenizer error")]
    TokenizerError(#[from] erl_tokenize::Error),
    #[error("expected other symbol")]
    WrongSymbol { expected: Symbol, got: Symbol },
    #[error("number too large for i64")]
    InvalidInteger,
}

pub enum Value {
    Integer(i64),
    Float(f64),
    // Boolean(bool),
    Atom(String),
    Binary(String),
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

pub fn consult<S: AsRef<str>>(s: S) -> Result<Vec<Value>, Error> {
    let mut res = Vec::new();
    let mut lexer = Lexer::new(s).peekable();

    loop {
        if lexer.peek().is_none() {
            break;
        }
        let form = parse(&mut lexer)?;
        lexer.expect_symbol(Symbol::Dot)?;
        res.push(form);
    }

    Ok(res)
}

fn parse<T: AsRef<str>>(l: &mut Peekable<Lexer<T>>) -> Result<Value, Error> {
    let next = l.next_token()?;
    dbg!(next.clone());
    match next {
        LexicalToken::Keyword(_) => Err(Error::InvalidToken(next)),
        LexicalToken::Variable(_) => Err(Error::InvalidToken(next)),
        LexicalToken::Atom(t) => Ok(Value::Atom(t.value().to_string())),
        LexicalToken::Char(t) => Ok(Value::Integer(t.value() as i64)),
        LexicalToken::Float(t) => Ok(Value::Float(t.value())),
        LexicalToken::Integer(t) => Ok(Value::Integer(
            t.value().try_into().map_err(|_| Error::InvalidInteger)?,
        )),
        LexicalToken::String(t) => Ok(Value::List(
            t.value()
                .chars()
                .map(|c| Value::Integer(c as i64))
                .collect(),
        )),
        LexicalToken::Symbol(ref t) => match t.value() {
            Symbol::OpenSquare => {
                dbg!("Open bracket");
                let mut res = Vec::new();
                if l.peek_symbol()? == Symbol::CloseSquare {
                    dbg!("Empty");
                    let consumed = l.next();
                    dbg!(consumed);
                    return Ok(Value::List(res));
                }
                loop {
                    res.push(parse(l)?);
                    match l.consume_symbol()? {
                        Symbol::Comma => continue,
                        Symbol::CloseSquare => break,
                        other => {
                            return Err(Error::WrongSymbol {
                                expected: Symbol::CloseSquare,
                                got: other,
                            })
                        }
                    }
                }
                Ok(Value::List(res))
            }
            Symbol::OpenBrace => {
                let mut res = Vec::new();
                if l.peek_symbol()? == Symbol::CloseBrace {
                    let _ = l.next();
                    return Ok(Value::Tuple(res));
                }
                loop {
                    res.push(parse(l)?);
                    match l.consume_symbol()? {
                        Symbol::Comma => continue,
                        Symbol::CloseBrace => break,
                        other => {
                            return Err(Error::WrongSymbol {
                                expected: Symbol::CloseBrace,
                                got: other,
                            })
                        }
                    }
                }
                Ok(Value::Tuple(res))
            }
            Symbol::Sharp => {
                let mut res = Vec::new();
                l.expect_symbol(Symbol::OpenBrace)?;
                if l.peek_symbol()? == Symbol::CloseBrace {
                    let _ = l.next();
                    return Ok(Value::Map(res));
                }

                loop {
                    let lhs = parse(l)?;
                    l.expect_symbol(Symbol::DoubleRightArrow)?;
                    let rhs = parse(l)?;

                    res.push((lhs, rhs));
                    match l.consume_symbol()? {
                        Symbol::Comma => continue,
                        Symbol::CloseBrace => break,
                        other => {
                            return Err(Error::WrongSymbol {
                                expected: Symbol::CloseBrace,
                                got: other,
                            })
                        }
                    }
                }
                Ok(Value::Map(res))
            }
            Symbol::DoubleLeftAngle => {
                let next = l.next_token()?;
                match next {
                    LexicalToken::String(t) => {
                        l.expect_symbol(Symbol::DoubleRightAngle)?;
                        Ok(Value::Binary(t.to_string()))
                    }
                    LexicalToken::Symbol(ref t) => {
                        if t.value() == Symbol::DoubleRightAngle {
                            Ok(Value::Binary("".to_owned()))
                        } else {
                            Err(Error::InvalidToken(next))
                        }
                    }
                    _ => Err(Error::InvalidToken(next)),
                }
            }
            Symbol::DoubleRightAngle => todo!(),
            _ => Err(Error::InvalidToken(next)),
        },
    }
}

trait LexerExt {
    fn next_token(&mut self) -> Result<LexicalToken, Error>;
    fn expect_symbol(&mut self, symbol: Symbol) -> Result<(), Error>;
    fn peek_symbol(&mut self) -> Result<Symbol, Error>;
    fn consume_symbol(&mut self) -> Result<Symbol, Error>;
}

impl<T: AsRef<str>> LexerExt for Peekable<Lexer<T>> {
    fn next_token(&mut self) -> Result<LexicalToken, Error> {
        if let Some(next) = self.next() {
            next.map_err(|e| e.into())
        } else {
            Err(Error::EndOfInput)
        }
    }

    fn expect_symbol(&mut self, symbol: Symbol) -> Result<(), Error> {
        match self.next_token()? {
            LexicalToken::Symbol(t) => {
                if t.value() == symbol {
                    Ok(())
                } else {
                    Err(Error::WrongSymbol {
                        expected: symbol,
                        got: t.value(),
                    })
                }
            }
            other => Err(Error::InvalidToken(other)),
        }
    }

    fn peek_symbol(&mut self) -> Result<Symbol, Error> {
        if let Some(next) = self.peek() {
            if let LexicalToken::Symbol(t) = next.clone().map_err(|e| Error::TokenizerError(e))? {
                Ok(t.value())
            } else {
                if let Ok(t) = next {
                    Err(Error::InvalidToken(t.clone()))
                } else {
                    Err(Error::InvalidInteger)
                }
            }
        } else {
            Err(Error::EndOfInput)
        }
    }

    fn consume_symbol(&mut self) -> Result<Symbol, Error> {
        let peek_res = self.peek_symbol();
        if peek_res.is_ok() {
            let _ = self.next();
        }
        peek_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        consult("[].").unwrap();
        consult("#{}.").unwrap();
        consult("{}.").unwrap();
        consult("<<>>.").unwrap();
    }

    #[test]
    fn test_consult() {
        let res = consult(
            r#"
            {<<"app">>,<<"katana_code">>}.
            {<<"build_tools">>,[<<"rebar3">>]}.
            {<<"description">>,<<"Functions useful for processing Erlang code.">>}.
            {<<"files">>,
               [<<"LICENSE">>,<<"README.md">>,<<"rebar.config">>,<<"rebar.lock">>,
                <<"src/katana_code.app.src">>,<<"src/ktn_code.erl">>,
                <<"src/ktn_dodger.erl">>,<<"src/ktn_io_string.erl">>]}.
            {<<"licenses">>,[<<"Apache 2.0">>]}.
            {<<"links">>,[{<<"Github">>,<<"https://github.com/inaka/katana-code">>}]}.
            {<<"name">>,<<"katana_code">>}.
            {<<"requirements">>,[]}.
            {<<"version">>,<<"0.2.1">>}.
        "#,
        );

        res.unwrap();
    }
}
