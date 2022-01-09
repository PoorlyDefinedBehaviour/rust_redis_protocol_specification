/// Sending commands to a Redis Server
///
/// How the interaction between the client and the server works:
///
/// A client sends the Redis server a RESP Array consisting of just Bulk Strings.
/// A Redis server replies to clients sending any valid RESP data type as reply.
///
/// # Examples
///
/// The client sends the command LLEN mylist in order to get the length
/// of the list stored at key mylist, and the server replies with an integer
/// reply:
///
/// ```terminal
/// client: "*2\r\n$4\r\nLLEN\r\n$6mylist\r\n" -- the request
/// server: ":48293\r\n"                       -- the reply
/// ```
use miette::{IntoDiagnostic, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

use crate::data_type::DataType;
use crate::resp;

#[derive(Debug)]
pub struct Redis {
  stream: TcpStream,
}

#[derive(Debug, PartialEq)]
pub enum Reply {
  Error(String),
  Ok(DataType),
}

impl Redis {
  pub async fn connect(ip: &str) -> Result<Self> {
    info!(ip, "connecting");

    let stream = TcpStream::connect(ip).await.into_diagnostic()?;

    info!(ip, "connected");

    Ok(Self { stream })
  }

  async fn send_request(&mut self, command: &str) -> Result<Reply> {
    info!(command, "sending command");

    self
      .stream
      .write_all(command.as_bytes())
      .await
      .into_diagnostic()?;

    let mut buffer = vec![0; 4096];

    let _bytes_read = self.stream.read(&mut buffer).await.into_diagnostic()?;

    info!("reply: {}", String::from_utf8_lossy(&buffer));

    match resp::parse(buffer)? {
      DataType::Error(message) => Ok(Reply::Error(message)),
      data_type => Ok(Reply::Ok(data_type)),
    }
  }

  pub async fn send(&mut self, command: &str) -> Result<Reply> {
    info!(command, "sending command");

    let encoded_command = resp::encode(command)?;

    self.send_request(&encoded_command).await
  }

  #[allow(dead_code)]
  pub async fn flushall(&mut self) -> Result<Reply> {
    self.send_request("FLUSHALL\r\n").await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const TEST_REDIS_IP: &'static str = "127.0.0.1:6380";

  #[tokio::test]
  async fn basic_commands() -> Result<()> {
    let mut redis = Redis::connect(TEST_REDIS_IP).await?;

    assert_eq!(
      Reply::Ok(DataType::SimpleString(String::from("OK"))),
      redis.flushall().await?,
    );

    assert_eq!(
      Reply::Ok(DataType::Int(0)),
      redis.send("LLEN mylist").await?,
    );

    assert_eq!(
      Reply::Ok(DataType::Int(1)),
      redis.send(r#"LPUSH mylist World"#).await?
    );

    assert_eq!(
      Reply::Ok(DataType::Int(2)),
      redis.send(r#"LPUSH mylist Hello"#).await?
    );

    assert_eq!(
      Reply::Ok(DataType::Int(2)),
      redis.send("LLEN mylist").await?,
    );

    assert_eq!(
      Reply::Ok(DataType::BulkString(String::from("Hello"))),
      redis.send("LPOP mylist").await?,
    );

    assert_eq!(
      Reply::Ok(DataType::BulkString(String::from("World"))),
      redis.send("LPOP mylist").await?,
    );

    assert_eq!(
      Reply::Ok(DataType::Int(0)),
      redis.send("LLEN mylist").await?
    );

    Ok(())
  }
}
