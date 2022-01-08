/// While the Redis protocol is very human readable and
/// easy to implement it can be implemented with a performance similar to that of a binary protocol.
///
/// RESP uses prefixed lengths to transfer bulk data,
/// so there is never a need to scan the payload for special characters like it happens for instance with JSON,
/// nor to quote the payload that needs to be sent to the server.
use crate::data_type::DataType;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ParserError {
  #[error("unexpected byte at position {0}")]
  UnexpectedByte(usize),
  #[error("the input ended unexpectedly")]
  UnexpectedEndOfInput,
  #[error("expected type {expected} but got {got}")]
  UnexpectedType { expected: String, got: String },
  #[error("expected value to be {expected} but got {got}")]
  UnexpectedValue { expected: String, got: String },
}

#[derive(Debug)]
struct Parser {
  /// The current position we are looking at in `input`.
  position: usize,
  input: Vec<u8>,
}

impl Parser {
  fn new(input: Vec<u8>) -> Self {
    Self { input, position: 0 }
  }

  /// Advances the current position by 1.
  fn skip(&mut self) {
    self.position += 1;
  }

  /// Returns the input byte at the current position.
  ///
  /// The current position is advanced by 1.
  fn next_byte(&mut self) -> Option<u8> {
    let byte = self.input.get(self.position);
    self.position += 1;
    byte.cloned()
  }

  /// Returns true if the parser has not reached the end of `input`.
  fn has_bytes_to_parse(&self) -> bool {
    self.position < self.input.len() - 1
  }

  /// Returns true when `position` points to the start of a termination: "\r\n"
  fn is_at_crlf(&self) -> bool {
    // "\r\n" occupies two bytes, if we don't have two bytes to look at,
    // we know we aren't at a termination.
    if self.position > self.input.len() - 2 {
      return false;
    }

    return self.input[self.position] == b'\r' && self.input[self.position + 1] == b'\n';
  }

  /// Tries to consume the crlf the parser is currently looking at.
  ///
  /// Returns error if the parser is not looking at a crlf.
  fn consume_crlf(&mut self) -> Result<(), ParserError> {
    if !self.is_at_crlf() {
      Err(ParserError::UnexpectedByte(self.position))
    } else {
      // Skip "\r".
      self.skip();
      // Skip "\n".
      self.skip();

      Ok(())
    }
  }

  fn data_type(&mut self) -> Result<DataType, ParserError> {
    match self.next_byte() {
      None => Err(ParserError::UnexpectedEndOfInput),
      Some(byte) => match byte {
        b'+' => self.simple_string(),
        b'$' => self.bulk_string_or_null(),
        b'-' => self.error(),
        b':' => self.int(),
        b'*' => self.array_or_null(),
        _ => todo!(),
      },
    }
  }

  fn simple_string(&mut self) -> Result<DataType, ParserError> {
    let string_starts_at = self.position;

    while self.has_bytes_to_parse() && !self.is_at_crlf() {
      self.skip();
    }

    let string = DataType::SimpleString(
      String::from_utf8_lossy(&self.input[string_starts_at..self.position]).to_string(),
    );

    self.consume_crlf()?;

    Ok(string)
  }

  /// Parses a RESP Bulk String.
  fn bulk_string_or_null(&mut self) -> Result<DataType, ParserError> {
    let string_length = self.parse_int()?;

    self.consume_crlf()?;

    if string_length == -1 {
      return Ok(DataType::Null);
    }

    let string_starts_at = self.position;

    for _ in 0..string_length {
      self.skip();
    }

    let string = DataType::BulkString(
      String::from_utf8_lossy(&self.input[string_starts_at..self.position]).to_string(),
    );

    self.consume_crlf()?;

    Ok(string)
  }

  fn error(&mut self) -> Result<DataType, ParserError> {
    let error_starts_at = self.position;

    while self.has_bytes_to_parse() && !self.is_at_crlf() {
      self.skip();
    }

    let error = DataType::Error(
      String::from_utf8_lossy(&self.input[error_starts_at..self.position]).to_string(),
    );

    self.consume_crlf()?;

    Ok(error)
  }

  fn parse_int(&mut self) -> Result<i64, ParserError> {
    let int_starts_at = self.position;

    while self.has_bytes_to_parse() && !self.is_at_crlf() {
      self.skip();
    }

    let lexeme = String::from_utf8_lossy(&self.input[int_starts_at..self.position]).to_string();

    match lexeme.parse::<i64>() {
      Err(_) => Err(ParserError::UnexpectedType {
        expected: String::from("int"),
        got: lexeme,
      }),
      Ok(i) => Ok(i),
    }
  }

  fn int(&mut self) -> Result<DataType, ParserError> {
    let int = self.parse_int()?;

    self.consume_crlf()?;

    Ok(DataType::Int(int))
  }

  fn array_or_null(&mut self) -> Result<DataType, ParserError> {
    let array_length = self.parse_int()?;

    self.consume_crlf()?;

    if array_length == -1 {
      return Ok(DataType::Null);
    }

    if array_length < 0 {
      return Err(ParserError::UnexpectedValue {
        expected: String::from("positive integer"),
        got: array_length.to_string(),
      });
    }

    let mut elements = Vec::with_capacity(array_length as usize);

    for _ in 0..array_length as usize {
      elements.push(self.data_type()?);
    }

    Ok(DataType::Array(elements))
  }
}

pub fn parse(input: Vec<u8>) -> Result<DataType, ParserError> {
  Parser::new(input).data_type()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
  }

  #[test]
  fn simple_string() {
    let tests = vec![("+OK\r\n", Ok(DataType::SimpleString(String::from("OK"))))];

    for (input, expected) in tests {
      let actual = parse(bytes(input));
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn error() {
    let tests = vec![(
      "-ERR unknown command 'foobar'\r\n",
      Ok(DataType::Error(String::from(
        "ERR unknown command 'foobar'",
      ))),
    )];

    for (input, expected) in tests {
      let actual = parse(bytes(input));
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn int() {
    let tests = vec![
      (":0\r\n", Ok(DataType::Int(0))),
      (":1000\r\n", Ok(DataType::Int(1000))),
      (":-3\r\n", Ok(DataType::Int(-3))),
    ];

    for (input, expected) in tests {
      let actual = parse(bytes(input));
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn bulk_string() {
    let tests = vec![
      ("$0\r\n\r\n", Ok(DataType::BulkString(String::new()))),
      (
        "$6\r\nfoobar\r\n",
        Ok(DataType::BulkString(String::from("foobar"))),
      ),
    ];

    for (input, expected) in tests {
      let actual = parse(bytes(input));
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn array() {
    let tests = vec![
      ("*0\r\n", Ok(DataType::Array(vec![]))),
      (
        "*3\r\n:1\r\n:2\r\n:3\r\n",
        Ok(DataType::Array(vec![
          DataType::Int(1),
          DataType::Int(2),
          DataType::Int(3),
        ])),
      ),
      (
        "*3\r\n$3\r\nfoo\r\n:1\r\n:2\r\n",
        Ok(DataType::Array(vec![
          DataType::BulkString(String::from("foo")),
          DataType::Int(1),
          DataType::Int(2),
        ])),
      ),
    ];

    for (input, expected) in tests {
      let actual = parse(bytes(input));
      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn null() {
    let tests = vec!["$-1\r\n", "*-1\r\n"];

    for input in tests {
      let actual = parse(bytes(input));
      assert_eq!(Ok(DataType::Null), actual);
    }
  }
}
