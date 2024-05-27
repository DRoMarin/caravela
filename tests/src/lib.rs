#[cfg(test)]
mod tests {
    use std::error::Error;

    use caravela::agent::*;
    use caravela::*;
    //use caravel_derive::*;

    #[test]
    fn platform_boot() -> Result<(), Box<dyn Error>> {
        struct Test(Agent);
        impl Behavior for Test {
            fn action(&mut self) {
                println!(
                    "[TEST] {}: Hello! I'm Agent Test",
                    self.agent_mut_ref().aid()
                )
            }

            fn agent_mut_ref(&mut self) -> &mut Agent {
                &mut self.0
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }
        }
        let mut agent_platform = Platform::new("test_boot");
        let boot = agent_platform.boot();
        assert!(boot.is_ok());
        std::thread::sleep(std::time::Duration::from_millis(5000));
        let agent_test = agent_platform.add::<Test>("AgentTest", 1, 4)?;
        let start = agent_platform.start(&agent_test);
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(start.is_ok());
        Ok(())
    }

    #[test]
    fn instantiating() -> Result<(), Box<dyn Error>> {
        struct Valid(Agent);
        struct Invalid {
            ag: Agent,
        }

        impl Behavior for Valid {
            fn action(&mut self) {}

            fn agent_mut_ref(&mut self) -> &mut Agent {
                &mut self.0
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }
        }
        impl Behavior for Invalid {
            fn action(&mut self) {}

            fn agent_mut_ref(&mut self) -> &mut Agent {
                &mut self.ag
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self { ag: base_agent }
            }
        }
        let mut agent_platform = Platform::new("test_inst");
        agent_platform.boot()?;
        let ag_valid = agent_platform.add::<Valid>("Agent-Valid", 98, 4);
        assert!(ag_valid.is_ok());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let ag_invalid = agent_platform.add::<Invalid>("Agent-Invalid", 99, 4);
        assert!(ag_invalid.is_err());
        Ok(())
    }

    #[test]
    fn contacts() -> Result<(), Box<dyn Error>> {
        struct AgentList(Agent);
        struct AgentPresent(Agent);

        impl Behavior for AgentPresent {
            fn action(&mut self) {
                println!("[TEST] {}: Waiting for message...", self.0.aid());
                self.0.wait(5000);
                _ = self.0.receive();
                println!("[TEST] {}: Received message!", self.0.aid());
            }

            fn failure_detection(&mut self) -> bool {
                false
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }

            fn agent_mut_ref(&mut self) -> &mut Agent {
                &mut self.0
            }
        }

        impl Behavior for AgentList {
            fn setup(&mut self) {
                println!("[TEST] {}: Adding contact to list", self.0.aid());
                let result: Result<(), ErrorCode> = self.0.add_contact("Agent-Present");
                assert_eq!(result, Ok(()), "NOT ADDED CORRECTLY");
                println!("[TEST] {}: Added {} as contact", self.0.aid(), "Agent-Present");
                let result = self.0.add_contact("Agent-Absent");
                assert_eq!(
                    result,
                    Err(ErrorCode::AidHandleNone),
                    "AGENT IS NOT MISSING"
                );
                println!("[TEST] {}: Contact {} not added", self.0.aid(), "Agent-Absent");
                self.0.set_msg(MessageType::Inform, Content::None);
                let _ = self.0.send_to("Agent-Present");
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }

            fn agent_mut_ref(&mut self) -> &mut Agent {
                &mut self.0
            }
        }

        let mut agent_platform = Platform::new("test_contacts");
        agent_platform.boot()?;

        let ag_present = agent_platform.add::<AgentPresent>("Agent-Present", 1, 10)?;
        let ag_list = agent_platform.add::<AgentList>("Agent-List", 1, 10)?;
        println!("STARTING PRESENT");
        agent_platform.start(&ag_present)?;
        println!("STARTING LIST");
        agent_platform.start(&ag_list)?;
        std::thread::sleep(std::time::Duration::from_millis(15000));
        Ok(())
    }

    /*
        #[test]
        fn concurrent() {
            struct AgentFast {
                rate: u64,
            }
            struct AgentSlow {
                rate: u64,
            }
            let data_fast = AgentFast { rate: 1 };
            let data_slow = AgentSlow { rate: 2 };

            impl Behavior for Agent<AgentFast> {
                fn action(&mut self) {
                    for _ in 0..6 {
                        println!("beep");
                        self.wait(self.data.rate * 1000);
                    }
                }
                fn failure_detection(&mut self) -> bool {
                    false
                }
            }

            impl Behavior for Agent<AgentSlow> {
                fn action(&mut self) {
                    for _ in 0..3 {
                        println!("boop");
                        self.wait(self.data.rate * 1000);
                    }
                }
                fn failure_detection(&mut self) -> bool {
                    false
                }
            }

            //let _ = scheduler::set_self_policy(scheduler::Policy::Fifo, 0);
            let mut agent_platform = Platform::new("test_concurrent".to_string());
            agent_platform.boot();
            let ag_fast = agent_platform
                .add("Agent-Fast".to_string(), 1, 10, data_fast)
                .unwrap();
            let ag_slow = agent_platform
                .add("Agent-Slow".to_string(), 1, 10, data_slow)
                .unwrap();
            println!("STARTING FAST");
            agent_platform.start(ag_fast);
            println!("STARTING SLOW");
            agent_platform.start(ag_slow);
            std::thread::sleep(std::time::Duration::from_millis(8000));
        }
        /*
            //#[test]
            fn ping_pong() {
                struct Player {
                    delay: u64,
                    event: &'static str,
                    target: &'static str,
                }
                let ping_data = Player {
                    delay: 500,
                    event: "ping!",
                    target: "Agent-Pong",
                };
                let pong_data = Player {
                    delay: 500,
                    event: "pong!",
                    target: "Agent-Ping",
                };

                impl Behavior for Agent<Player> {
                    fn setup(&mut self) {
                        loop {
                            let result = self.add_contact(self.data.target);
                            if result.is_ok() {
                                println!("FOUND");
                                break;
                            }
                        }
                        if self.get_nickname() == "Agent-Ping" {
                            self.msg.set_type(MessageType::Inform);
                            self.msg
                                .set_content(Content::Text(self.data.event.to_string()));

                            println!("STARTING {}\n", self.data.event);
                            loop {
                                let send_result = self.send_to("Agent-Pong");
                                if send_result.is_ok() {
                                    break;
                                }
                                self.wait(self.data.delay);
                                println!("RETRY {}\n", self.data.event);
                            }
                        }
                    }

                    fn action(&mut self) {
                        if self.receive() == Ok(MessageType::Inform) {
                            if let Some(Content::Text(x)) = self.msg.get_content() {
                                println!("{}", x);
                                self.wait(self.data.delay);
                            }
                            self.msg.set_type(MessageType::Inform);
                            self.msg
                                .set_content(Content::Text(self.data.event.to_string()));
                            self.send_to(self.data.target);
                        }
                    }

                    fn done(&mut self) -> bool {
                        false
                    }
                    fn failure_detection(&mut self) -> bool {
                        false
                    }
                }

                let mut agent_platform = Platform::new("demo".to_string());
                agent_platform.boot();
                let ag_ping = agent_platform
                    .add("Agent-Ping".to_string(), 1, 10, ping_data)
                    .unwrap();
                let ag_pong = agent_platform
                    .add("Agent-Pong".to_string(), 1, 10, pong_data)
                    .unwrap();

                agent_platform.start(ag_pong);

                agent_platform.start(ag_ping);
                std::thread::sleep(std::time::Duration::from_millis(10000));
            }
        */
    */
}
