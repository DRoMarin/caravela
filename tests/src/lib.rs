#[cfg(test)]
mod tests {
    use caravel::*;
    //use caravel_derive::*;

    #[test]
    fn platform_boot() {
        struct Test(Agent);
        impl Behavior for Test {
            fn action(&mut self) {
                self.get_agent_ref().get_nickname();
                println!(
                    "\n{}: Hello! I'm Agent A",
                    self.get_agent_ref().get_aid().get_name()
                )
            }

            fn get_agent_ref(&mut self) -> &mut Agent {
                &mut self.0
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }
        }
        let mut agent_platform = Platform::new("test_boot".to_string());
        let boot = agent_platform.boot();
        assert!(boot.is_ok());
        let agent: Test = agent_platform.add("Agent-A".to_string(), 1, 4).unwrap();
        let start = agent_platform.start(agent);
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(start.is_ok());
    }

    #[test]
    fn instantiating() {
        struct Valid(Agent);
        struct Invalid(Agent);

        impl Behavior for Valid {
            fn action(&mut self) {}

            fn get_agent_ref(&mut self) -> &mut Agent {
                &mut self.0
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }
        }
        impl Behavior for Invalid {
            fn action(&mut self) {}

            fn get_agent_ref(&mut self) -> &mut Agent {
                &mut self.0
            }

            fn agent_builder(base_agent: Agent) -> Self {
                Self(base_agent)
            }
        }
        let mut agent_platform = Platform::new("test_inst".to_string());
        let _ = agent_platform.boot();
        let ag_b: Result<Valid, &str> = agent_platform.add("Agent-Valid".to_string(), 98, 4);
        assert!(ag_b.is_ok());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let ag_c: Result<Invalid, &str> = agent_platform.add("Agent-Invalid".to_string(), 99, 4);
        assert!(ag_c.is_err());
    }

    /*
    #[test]
    fn contacts() {
        struct AgentList(Agent<()>, u32);
        struct AgentPresent(Agent<()>);
        let data_list = AgentList;
        let data_present = AgentPresent;

        impl Behavior for AgentPresent {
            fn action(&mut self) {
                println!("{}: waiting", self.get_nickname());
                self.wait(5000);
                self.receive();
            }

            fn failure_detection(&mut self) -> bool {
                false
            }
        }

        impl Behavior for AgentList {
            fn setup(&mut self) {
                println!("ADDING CONTACTS");
                let result = self.add_contact("Agent-Present");
                assert_eq!(result, Ok(()), "NOT ADDED CORRECTLY");
                let result = self.add_contact("Agent-Absent");
                assert_eq!(
                    result,
                    Err(ErrorCode::NotRegistered),
                    "AGENT IS NOT MISSING"
                );
                println!("ADDED CONTACTS");
                self.set_msg(MessageType::Inform, Content::None);
                self.send_to("Agent-Present");
            }
        }

        //let _ = scheduler::set_self_policy(scheduler::Policy::Fifo, 0);

        let mut agent_platform = Platform::new("test_contacts".to_string());
        agent_platform.boot();
        let ag_present = agent_platform
            .add("Agent-Present".to_string(), 1, 10, data_present)
            .unwrap();

        let ag_list = agent_platform
            .add("Agent-List".to_string(), 1, 10, data_list)
            .unwrap();

        println!("STARTING PRESENT");
        agent_platform.start(ag_present);
        println!("STARTING LIST");
        agent_platform.start(ag_list);
        std::thread::sleep(std::time::Duration::from_millis(15000));
    }
    */
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