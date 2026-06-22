use crate::Context;

pub trait CliCommand {
    fn run(&self, ctx: &Context);
}

pub fn execute(cmd: impl CliCommand) {
    crate::output::init_logging();
    let ctx = Context { verbose: true };

    cmd.run(&ctx);
}
