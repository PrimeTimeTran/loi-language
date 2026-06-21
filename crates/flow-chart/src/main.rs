use flow_chart::config::Config;
use flow_chart::evaluator::Evaluator;

fn main() {
    let config = Config::load();

    let mut evaluator = Evaluator::new(config);
    evaluator.evaluate_fs();
}
