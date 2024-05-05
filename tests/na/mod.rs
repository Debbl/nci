use nci::{agents::Agent, parse::parse_na};

use crate::common::expect;
pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn na(agent: Agent, args: Vec<String>, expected: String) {
    expect(parse_na, agent, args, expected);
}
