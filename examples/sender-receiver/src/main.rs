/*****************************
 *  sender receiver example  *
 *****************************/

// Importing crate components
use caravela::{
    agent::{Agent, AgentBuild},
    behavior::Behavior,
    caravela_probe, make_agent,
    messaging::{Content, MessageType},
    ErrorCode, Platform, DEFAULT_STACK,
};
use std::error::Error;

//Defining agent types
make_agent!(Sender);
make_agent!(Receiver);

//implementing behaviors for each type
impl Behavior for Sender {
    fn setup(&mut self) -> Result<(), ErrorCode> {
        self.agent.add_contact("AgentReceiver")
    }
    fn action(&mut self) -> Result<(), ErrorCode> {
        caravela_probe!("{}: Hello! I'm Agent Sender", self.agent.name());
        self.agent.send_to_all(
            MessageType::Inform,
            Content::Expression("This is a message".to_string()),
        )?;
        self.agent.wait(200);
        Ok(())
    }

    fn done(&mut self) -> bool {
        true
    }
}

impl Behavior for Receiver {
    fn action(&mut self) -> Result<(), ErrorCode> {
        caravela_probe!("{}: Hello! I'm Agent Receiver", self.agent.name());
        let result = self.agent.receive();
        if let Ok(msg) = result {
            if let Content::Expression(text) = msg.content() {
                println!("msg: {}", text);
            }
        }
        Ok(())
    }

    fn done(&mut self) -> bool {
        true
    }
}

// main entry
fn main() -> Result<(), Box<dyn Error>> {
    // new platform
    let agent_platform = Platform::new("example")?;
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
