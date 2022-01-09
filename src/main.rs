use crate::redis::Redis;

/// Implementando o Redis Protocol specification
/// ///
/// RESP protocol description
///
/// The RESP protocol was introduced in Redis 1.2, but it became the standard way for talking with the Redis server in Redis 2.0. This is the protocol you should implement in your Redis client.
///
/// RESP is actually a serialization protocol that supports the following data types: Simple Strings, Errors, Integers, Bulk Strings and Arrays.
///
/// The way RESP is used in Redis as a request-response protocol is the following:
///
///     Clients send commands to a Redis server as a RESP Array of Bulk Strings.
///     The server replies with one of the RESP types according to the command implementation.
///
/// In RESP, the type of some data depends on the first byte:
///
///     For Simple Strings the first byte of the reply is "+"
///     For Errors the first byte of the reply is "-"
///     For Integers the first byte of the reply is ":"
///     For Bulk Strings the first byte of the reply is "$"
///     For Arrays the first byte of the reply is "*"
///
/// In RESP different parts of the protocol are always terminated with "\r\n" (CRLF).
mod data_type;
mod redis;
mod resp;

use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
  std::env::set_var(
    "RUST_LOG",
    std::env::var("RUST_LOG").unwrap_or(String::from("redis=trace")),
  );

  tracing_subscriber::fmt::init();

  let mut redis = Redis::connect("127.0.0.1:6379").await?;

  dbg!(redis.send("LLEN mylist").await);

  Ok(())
}
