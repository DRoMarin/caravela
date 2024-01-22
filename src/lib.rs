pub mod platform;

#[cfg(test)]
mod tests {
    use crate::platform::{
        agent::{
            behavior::{private::TaskControl, Behavior},
            Agent,
        },
        entity::{messaging::MessageType, Entity},
        ErrorCode, Platform,
    };

    //#[test]
    fn platform_boot() {
        struct A;
        let data_a = A;
        impl Behavior for Agent<A> {
            fn action(&mut self) {
                self.get_nickname();
                println!("\n{}: Hello! I'm Agent A", self.hub.aid.get_name())
            }
        }
        let mut agent_platform = Platform::new("test_boot".to_string());
        let boot = agent_platform.boot();
        assert!(boot.is_ok());
        let ag_a = agent_platform
            .add("Agent-A".to_string(), 1, 4, data_a)
            .unwrap();
        let start = agent_platform.start(ag_a);
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(start.is_ok());
    }

    //#[test]
    fn instantiating() {
        struct Valid;
        struct Invalid;
        let data_valid = Valid;
        let data_invalid = Invalid;

        impl Behavior for Agent<Valid> {
            fn action(&mut self) {}
        }
        impl Behavior for Agent<Invalid> {
            fn action(&mut self) {}
        }
        let mut agent_platform = Platform::new("test_inst".to_string());
        let _ = agent_platform.boot();

        let ag_b = agent_platform.add("Agent-Valid".to_string(), 98, 4, data_valid);
        assert!(ag_b.is_ok());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let ag_c = agent_platform.add("Agent-Invalid".to_string(), 99, 4, data_invalid);
        assert!(ag_c.is_err());
    }

    //#[test]
    fn contacts() {
        struct AgentList;
        struct AgentPresent;
        let data_list = AgentList;
        let data_present = AgentPresent;

        impl Behavior for Agent<AgentPresent> {
            fn action(&mut self) {
                println!("{}: waiting", self.get_nickname());
                self.wait(10000);
                self.receive();
            }

            fn failure_detection(&mut self) -> bool {
                false
            }
        }

        impl Behavior for Agent<AgentList> {
            fn setup(&mut self) {
                println!("ADDING CONTACTS");
                let result = self.add_contact("Agent-Present");
                assert_eq!(result, ErrorCode::NoError, "NOT ADDED CORRECTLY");
                let result = self.add_contact("Agent-Absent");
                assert_eq!(result, ErrorCode::NotRegistered, "AGENT IS NOT MISSING");
                println!("ADDED CONTACTS");
                self.msg.set_type(MessageType::Inform);
                self.send_to("Agent-Present");
            }
        }

        let _ = scheduler::set_self_policy(scheduler::Policy::Fifo, 0);

        let mut agent_platform = Platform::new("test_contacts".to_string());
        let _ = agent_platform.boot();
        let ag_present = agent_platform
            .add("Agent-Present".to_string(), 1, 10, data_present)
            .unwrap();
        let ag_list = agent_platform
            .add("Agent-List".to_string(), 1, 10, data_list)
            .unwrap();
        println!("STARTING PRESENT");
        let _ = agent_platform.start(ag_present);
        //std::thread::sleep(std::time::Duration::from_millis(1000));
        println!("STARTING LIST");
        let _ = agent_platform.start(ag_list);
        std::thread::sleep(std::time::Duration::from_millis(15000));
    }

    #[test]
    fn concurrent() {
        struct AgentFast {
            rate: u64,
        };
        struct AgentSlow {
            rate: u64,
        }
        let data_fast = AgentFast { rate: 1 };
        let data_slow = AgentSlow { rate: 2 };

        impl Behavior for Agent<AgentFast> {
            fn action(&mut self) {
                println!("beep");
                self.wait(self.data.rate * 1000);
            }
            fn done(&mut self) -> bool {
                false
            }
            fn failure_detection(&mut self) -> bool {
                false
            }
        }

        impl Behavior for Agent<AgentSlow> {
            fn action(&mut self) {
                println!("boop");
                self.wait(self.data.rate * 1000);
            }
            fn done(&mut self) -> bool {
                false
            }
            fn failure_detection(&mut self) -> bool {
                false
            }
        }

        let _ = scheduler::set_self_policy(scheduler::Policy::Fifo, 0);

        let mut agent_platform = Platform::new("test_concurrent".to_string());
        let _ = agent_platform.boot();
        let ag_fast = agent_platform
            .add("Agent-Fast".to_string(), 1, 10, data_fast)
            .unwrap();
        let ag_slow = agent_platform
            .add("Agent-Slow".to_string(), 1, 10, data_slow)
            .unwrap();
        println!("STARTING FAST");
        let _ = agent_platform.start(ag_fast);
        println!("STARTING SLOW");
        let _ = agent_platform.start(ag_slow);
        std::thread::sleep(std::time::Duration::from_millis(10000));
    }
}
