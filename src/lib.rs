pub mod platform;

#[cfg(test)]
mod tests {
    use crate::platform::{agent::{behavior::Behavior, Agent}, Platform};
    
    #[test]
    fn platform_boot(){
        let ap = Platform::new("crate_test".to_string());
        let boot = ap.boot();
        assert!(boot.is_ok());
    }

    #[test]
    fn instantiating() {
        struct A;
        struct B;
        let data_a = A;
        let data_b = B;

        impl<A> Behavior for Agent<A> {
            fn action(&mut self) {}
        }
        let ag_a = Agent::new("Agente-A".to_string(), 98, 4, "RPi", data_a);
        let ag_b = Agent::new("Agente-B".to_string(), 99, 4, "RPi", data_b);
        assert!(ag_a.is_ok());
        assert!(ag_b.is_err());
    }
}
