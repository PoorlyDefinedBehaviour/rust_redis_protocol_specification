# Basic [Redis protocol](https://redis.io/topics/protocol) implementation

Implements basic RESP encoding and parsing.

# The client

## Connecting to Redis

```rust
#[tokio::main]
async fn main() -> Result<()> {
  let mut redis = Redis::connect("127.0.0.1:6379").await?;

  Ok(())
}

```

## Sending commands to Redis

```rust
#[tokio::main]
async fn main() -> Result<()> {
  let mut redis = Redis::connect("127.0.0.1:6379").await?;

  match redis.send("LLEN mylist").await? {
    Reply::Error(e) => println!("ERROR: {}", e),
    Reply::Ok(data_type) => println!("OK: {:?}", data_type),
  };

  Ok(())
}

```

# The RESP parser

## Parsing Simple Strings

```rust
use crate::resp;

assert_eq!(
  resp::parse(b"+OK\r\n".to_vec()),
  Ok(DataType::SimpleString(String::from("OK"))),
)
```

## Parsing Error

```rust
use crate::resp;

assert_eq!(
  resp::parse(b"-ERR unknown command 'foobar'\r\n".to_vec()),
  Ok(DataType::Error(String::from(
    "ERR unknown command 'foobar'",
  ))),
)
```

## Parsing integers

```rust
use crate::resp;

assert_eq!(resp::parse(b":0\r\n".to_vec()), Ok(DataType::Int(0))),
assert_eq!(resp::parse(b":1000\r\n".to_vec()), Ok(DataType::Int(1000))),
assert_eq!(resp::parse(b":-3\r\n".to_vec()), Ok(DataType::Int(-3))),
```

## Parsing Bulk Strings

```rust
use crate::resp;

assert_eq!(
  resp::parse(b"$0\r\n\r\n".to_vec()),
  Ok(DataType::BulkString(String::new()))
)
assert_eq!(
  resp::parse(b"$6\r\nfoobar\r\n".to_vec()),
  Ok(DataType::BulkString(String::from("foobar"))),
)
```

## Parsing Arrays

```rust
use crate::resp;

assert_eq!(resp::parse(b"*0\r\n".to_vec()), Ok(DataType::Array(vec![])))

assert_eq!(
  resp::parse(b"*3\r\n:1\r\n:2\r\n:3\r\n".to_vec()),
  Ok(DataType::Array(vec![
      DataType::Int(1),
      DataType::Int(2),
      DataType::Int(3),
    ])
  ),
)
```

## Parsing Null

```rust
use crate::resp;

assert_eq!(resp::parse(b"$-1\r\n".to_vec()), Ok(DataType::Null))
assert_eq!(resp::parse(b"*-1\r\n".to_vec()), Ok(DataType::Null))
```

## The RESP encoder

Encoding basic commands

```rust
assert_eq!(
  resp::encode(&"LLEN mylist"),
  Ok(String::from("*2\r\n$4\r\nLLEN\r\n$6\r\nmylist\r\n"))
)
assert_eq!(
  resp::encode(&r#"SETEX mykey 10 "Hello""#),
  Ok(String::from("*4\r\n$5\r\nSETEX\r\n$5\r\nmykey\r\n:10\r\n$7\r\n\"Hello\"\r\n")),
)
```
