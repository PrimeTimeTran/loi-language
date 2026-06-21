use clap::{Parser, Subcommand};
use loid::{daemon::run::run, state};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start,

    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Start => {
            run().await;
        }

        Command::Status => {
            let mut stream = tokio::net::TcpStream::connect("127.0.0.1:7788")
                .await
                .unwrap();

            tokio::io::AsyncWriteExt::write_all(&mut stream, b"status")
                .await
                .unwrap();

            let mut buf = vec![];

            tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buf)
                .await
                .unwrap();

            println!("{}", String::from_utf8_lossy(&buf));
        }
    }
}
