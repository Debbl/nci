use nci::{agents::Agent, parse::parse_nlx};

use crate::common::expect;
pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn nlx(agent: Agent, args: Vec<String>, expected: String) {
    expect(parse_nlx, agent, args, expected);
}
