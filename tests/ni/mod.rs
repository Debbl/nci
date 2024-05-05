use nci::{agents::Agent, parse::parse_ni};

use crate::common::expect;

pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn ni(agent: Agent, args: Vec<String>, expected: String) {
    expect(parse_ni, agent, args, expected);
}
