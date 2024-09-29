/*****************************
 *  sender receiver example  *
 *****************************/

// Importing crate components
use caravela::{
    agent, agent::Agent, behavior::*, caravela_probe, messaging::*, Platform, DEFAULT_STACK,
};
use std::error::Error;
//Defining agent types
agent!(Sender);
agent!(Receiver);

//implementing behaviors for each type
impl Behavior for Sender {
    fn setup(&mut self) {
        self.agent().add_contact("AgentReceiver");
    }
    fn action(&mut self) {
        caravela_probe!("{}: Hello! I'm Agent Sender", self.agent().aid());
        self.agent().set_msg(
            MessageType::Inform,
            Content::Text("This is a message".to_string()),
        );
        self.agent().send_to_all();
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

        if let Content::Text(msg) = self.agent().msg().content() {
            println!("msg: {}", msg);
        }
    }

    fn done(&mut self) -> bool {
        false
    }
}

// main entry
fn main() -> Result<(), Box<dyn Error>> {
    // new platform
    let agent_platform = Platform::new("example");
    // boot
    agent_platform.boot()?;
    // add agents
    let agent_sender = agent_platform.add_agent::<Sender>("AgentSender", 1, DEFAULT_STACK)?;
    let agent_receiver = agent_platform.add_agent::<Receiver>("AgentReceiver", 2, DEFAULT_STACK)?;
    // start agents
    agent_platform.start(&agent_sender)?;
    agent_platform.start(&agent_receiver)?;
    // set program duration
    std::thread::sleep(std::time::Duration::from_millis(2000));
    Ok(())
}
