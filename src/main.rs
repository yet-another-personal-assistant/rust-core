use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures::stream::StreamExt;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

fn build_dst(channel: &str, src: &Value) -> Option<Value> {
    match channel {
        "tg" => Some(json!({"channel": "tg",
                            "chat_id": src["chat_id"]})),
        _ => None,
    }
}

fn reply(message: Value) -> io::Result<Vec<Value>> {
    let channel = if let Value::String(c) = &message["from"]["channel"] {
        c
    } else {
        eprintln!(
            "from-channel is not a string: {:?}",
            &message["from"]["channel"]
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "from-channel is not a string",
        ));
    };
    match build_dst(channel, &message["from"]) {
        Some(dst) => Ok(vec![json!({
            "text": ["Я не поняла",
                     format!("А что значит {}?", message["text"])],
            "to": dst,
            "from": {"channel": "brain",
                     "name": "niege"},
        })]),
        None => {
            println!("Can't build destination to reply to {}", message["from"]);
            Ok(vec![])
        }
    }
}

async fn handle(mut client: TcpStream) -> io::Result<()> {
    let (u_reader, mut writer) = client.split();
    let mut reader = BufReader::new(u_reader);
    let mut buffer = String::new();

    while reader.read_line(&mut buffer).await? > 0 {
        let message = serde_json::from_str(&buffer);
        match message {
            Ok(r) => match reply(r) {
                Ok(messages) => {
                    for message in messages {
                        let msg_str = format!("{}\n", serde_json::to_string(&message)?);
                        writer.write(msg_str.as_bytes()).await?;
                    }
                }
                Err(f) => {
                    println!("Error building replies: {}", f.to_string());
                }
            },
            Err(f) => {
                println!("Error parsing json: {}", f.to_string());
            }
        }
        buffer.clear();
    }

    Ok(())
}

async fn serve(mut listener: TcpListener) {
    let mut incoming = listener.incoming();
    while let Some(conn) = incoming.next().await {
        match conn {
            Err(e) => eprintln!("accept failed = {:?}", e),
            Ok(client) => {
                tokio::spawn(handle(client));
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6142);
    let listener = TcpListener::bind(addr).await.unwrap();

    let server = serve(listener);
    println!("Server running on localhost:6142");

    server.await;
}
