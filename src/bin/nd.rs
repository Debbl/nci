use nci::{parse::parse_nd, runner::run_cli};

fn main() {
    run_cli(parse_nd, None)
}
