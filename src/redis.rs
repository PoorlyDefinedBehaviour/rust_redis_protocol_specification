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

  pub async fn send(&mut self, command: &str) -> Result<Reply> {
    info!(command, "sending command");

    let encoded_command = resp::encode(command)?;

    info!(
      "sending RESP command: {}",
      &encoded_command.replace("\r", "\\r").replace("\n", "\\n")
    );

    self
      .stream
      .write_all(encoded_command.as_bytes())
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
}
