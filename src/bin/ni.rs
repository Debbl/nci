use nci::{parse::parse_ni, runner::run_cli};

fn main() {
    run_cli(parse_ni, None)
}
