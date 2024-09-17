use caravela::agent::*;
use caravela::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    agent!(Sender);
    agent!(Receiver);

    impl Behavior for Sender {
        fn action(&mut self) {
            caravela_probe!("{}: Hello! I'm Agent Sender", self.agent().aid());
            self.agent().send_to("AgentReceiver");
            self.agent().wait(200);
        }

        fn done(&mut self) -> bool {
            false
        }
    }

    impl Behavior for Receiver {
        fn action(&mut self) {
            self.agent().receive();
            caravela_probe!("{}: Hello! I'm Agent Receiver", self.agent().aid());
        }

        fn done(&mut self) -> bool {
            false
        }
    }

    let mut agent_platform = Platform::new("example");
    agent_platform.boot()?;
    let agent_sender = agent_platform.add_agent::<Sender>("AgentSender", 1, DEFAULT_STACK)?;
    let agent_receiver = agent_platform.add_agent::<Receiver>("AgentReceiver", 2, DEFAULT_STACK)?;
    agent_platform.start(&agent_sender)?;
    agent_platform.start(&agent_receiver)?;
    std::thread::sleep(std::time::Duration::from_millis(2000));
    Ok(())
}
