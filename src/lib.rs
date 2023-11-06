pub mod platform;

#[cfg(test)]
mod tests {
    use crate::platform::agent::{behavior::Behavior, Agent};

    #[test]
    fn instantiating() {
        struct A {}
        let dataA = A {};

        impl<A> Behavior for Agent<A> {
            fn action(&mut self) {}
        }
        let agA = Agent::new("Agente-A".to_string(), 1, 4, "RPi", dataA);
    }
}
