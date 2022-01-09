/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings and Arrays.
///
/// In RESP, the type of some data depends on the first byte:
///
/// For Simple Strings the first byte of the reply is "+"
/// For Errors the first byte of the reply is "-"
/// For Integers the first byte of the reply is ":"
/// For Bulk Strings the first byte of the reply is "$"
/// For Arrays the first byte of the reply is "*"
///
/// In RESP different parts of the protocol are always terminated with
/// "\r\n" (CRLF).
#[derive(Debug, PartialEq)]
pub enum DataType {
  /// When the first byte of the data is "+"
  ///
  /// Simple Strings are used to transmit non binary safe strings with minimal overhead.
  ///
  /// # Examples
  ///
  /// ```terminal
  /// "+OK\r\n"
  /// ```
  SimpleString(String),
  /// When the first byte of the data is "-"
  ///
  /// # Examples
  ///
  /// ```terminal
  /// "-ERR unknown command 'foobar'\r\n"
  /// ```
  Error(String),
  /// When the first byte of the data is ":"
  ///
  /// # Examples
  ///
  /// ```terminal
  /// ":0\r\n"
  /// ":1000\r\n"
  /// ```
  // TODO: is i64 enough?
  Int(i64),
  /// When the first byte of the data is "$"
  ///
  /// Bulk Strings are used in order to represent a single binary safe string up to 512 MB in length.
  ///
  /// Bulk Strings are encoded in the following way:
  ///
  /// A "$" byte followed by the number of bytes composing the string (a prefixed length), terminated by CRLF.
  /// The actual string data.
  /// A final CRLF.
  ///
  /// # Examples
  ///
  /// ```terminal
  /// "$6\r\nfoobar\r\n"
  /// ```
  ///
  /// The empty string is just:
  ///
  /// ```terminal
  /// "$0\r\n\r\n"
  /// ```
  BulkString(String),
  /// When the first byte of the data is "*"
  ///
  /// RESP Arrays are sent using the following format:
  ///
  /// A * character as the first byte, followed by the number of elements in the array as a decimal number, followed by CRLF.
  /// An additional RESP type for every element of the Array.
  ///
  /// Note that arrays can contain elements of different types.
  ///
  /// # Examples
  ///
  /// The empty array:
  ///
  /// ```terminal
  /// "*0\r\n"
  /// ```
  ///
  /// An array with two RESP Bulk Strings:
  ///
  /// ```terminal
  /// "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"
  /// ```
  ///
  /// An array of three integers:
  ///
  /// ```terminal
  /// "*3\r\n:1\r\n:2\r\n:3\r\n"
  /// ```
  ///
  /// An array with one string and two integers:
  ///
  /// ```terminal
  /// "*3\r\n$3\r\nfoo\r\n:1\r\n:2\r\n"
  /// ```
  Array(Vec<DataType>),
  /// When a Bulk String is used to signal non-existence of a value using
  /// a special format that is used to represent a Null value.
  ///
  /// A Bulk String with length equal to -1 and with no data
  /// represents a Null value.
  ///
  /// An array with length equal to -1 and no data also represents a Null value.
  ///
  /// It is called a Null Bulk String.
  ///
  /// # Examples
  ///
  /// ```terminal
  /// "$-1\r\n"
  /// "*-1\r\n"
  /// ```
  Null,
}
