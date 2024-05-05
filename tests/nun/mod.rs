use nci::{agents::Agent, parse::parse_nun};

use crate::common::expect;

pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;
pub mod yarn_berry;

pub fn nun(agent: Agent, args: Vec<String>, expected: String) {
    expect(parse_nun, agent, args, expected);
}
