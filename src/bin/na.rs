use nci::{parse::parse_na, runner::run_cli};

fn main() {
    run_cli(parse_na, None)
}
