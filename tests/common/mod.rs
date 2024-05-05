use nci::{agents::Agent, runner::Runner};

pub fn expect(func: Runner, agent: Agent, args: Vec<String>, expected: String) {
    let (agent, args) = func(agent, args, None);

    let mut args = args;
    args.insert(0, agent.to_string());
    let command = args;

    assert_eq!(command.join(" "), expected.as_str());
}
