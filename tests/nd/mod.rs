use nci::{agents::Agent, parse::parse_nd};

use crate::common::expect;

pub mod npm;
pub mod pnpm;
pub mod yarn_berry;

pub fn nd(agent: Agent, args: Vec<String>, expected: String) {
    expect(parse_nd, agent, args, expected);
}
