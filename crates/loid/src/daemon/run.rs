use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::state::state;

pub async fn run() {
    let mut s = state::load();

    s.starts += 1;
    s.started_at = state::now();

    state::save(&s);

    let listener = TcpListener::bind("127.0.0.1:7788").await.unwrap();

    println!("loid daemon running");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        let mut buf = [0; 1024];

        let n = socket.read(&mut buf).await.unwrap();

        let cmd = String::from_utf8_lossy(&buf[..n]);

        match cmd.trim() {
            "status" => {
                let s = state::load();

                let out = serde_json::to_string(&s).unwrap();

                socket.write_all(out.as_bytes()).await.unwrap();
            }

            _ => {}
        }
    }
}
