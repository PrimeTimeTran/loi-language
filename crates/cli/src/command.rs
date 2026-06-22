use crate::Context;

#[async_trait::async_trait]
pub trait CliCommand {
    async fn run(&self, ctx: &Context);
}

pub async fn execute(cmd: impl CliCommand, ctx: Context) {
    crate::output::init_logging();

    cmd.run(&ctx).await;
}
