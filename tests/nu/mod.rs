use nci::agents::Agent;

use crate::common::expect;

pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn nu(agent: Agent, args: Vec<String>, expected: String) {
    expect(nci::parse::parse_nu, agent, args, expected);
}
