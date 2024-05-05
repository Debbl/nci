use nci::{agents::Agent, parse::parse_nr};

use crate::common::expect;

pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn nr(agent: Agent, args: Vec<String>, expected: String) {
    expect(
        |agent, args, _ctx| parse_nr(agent, args),
        agent,
        args,
        expected,
    );
}
