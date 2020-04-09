use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "channel", rename_all = "snake_case")]
enum PaOrigin {
    Tg { chat_id: i64, user_id: Option<i64> },
    Brain { name: String },
}

fn build_dst(src: &PaOrigin) -> Option<PaOrigin> {
    match src {
        PaOrigin::Tg {
            chat_id,
            user_id: Some(user_id),
        } if chat_id == user_id => Some(PaOrigin::Tg {
            chat_id: *chat_id,
            user_id: None,
        }),
        _ => None,
    }
}

#[derive(Serialize, Deserialize)]
struct PaMsg {
    from: PaOrigin,
    to: PaOrigin,
    text: Value,
}

fn reply(message: PaMsg) -> Vec<PaMsg> {
    match build_dst(&message.from) {
        Some(dst) => vec![PaMsg {
            text: json!(["Я не поняла", format!("А что значит {}?", message.text)]),
            to: dst,
            from: PaOrigin::Brain {
                name: String::from("niege"),
            },
        }],
        None => {
            println!("Can't build destination to reply to {:?}", message.from);
            vec![]
        }
    }
}

async fn handle(mut client: TcpStream) -> io::Result<()> {
    let (u_reader, mut writer) = client.split();
    let mut reader = BufReader::new(u_reader);
    let mut buffer = String::new();

    while reader.read_line(&mut buffer).await? > 0 {
        let messages = serde_json::from_str(&buffer)
            .map(reply)
            .unwrap_or_else(|f| {
                println!("Error parsing json: {}", f.to_string());
                vec![]
            });
        for message in messages {
            writer.write(&serde_json::to_vec(&message)?).await?;
            writer.write(b"\n").await?;
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
