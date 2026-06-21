use loid::{
    cli::command::{Command, parse},
    daemon::{run::run, status::status},
};

#[tokio::main]
async fn main() {
    let cli = parse();

    match cli.command {
        Command::Start => run().await,
        Command::Status => status().await,
        // Command::View => view().new().await,
        // Command::ViewFork => view().create.await,
        // Command::Explain => explain().new.await,
        // Command::ExplainDoc => explain().new().doc().await,

        // Command::Start        → daemon::start()
        // Command::Status       → daemon::status()
        // Command::View         → view::set_active(...)
        // Command::ViewFork     → view::fork(...)
        // Command::Explain      → explain::run(...)
        // Command::ExplainDoc   → explain::doc(...)
        _ => {
            println!("all done")
        }
    }
}
