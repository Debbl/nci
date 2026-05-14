use nci::{
    parse::parse_ni,
    runner::{run_cli, DetectOptions},
};

fn main() {
    run_cli(
        |agent, mut args, ctx| {
            args.push("--frozen-if-present".into());
            parse_ni(agent, args, ctx)
        },
        Some(DetectOptions::new().with_auto_install(true)),
    )
}
