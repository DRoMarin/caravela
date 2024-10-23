/// Macro used for simple messages activated by the user as a feature.
#[macro_export]
#[cfg(feature = "dbg-probe")]
macro_rules! caravela_probe {
    ($($arg:tt)*) => {{
    println!("[PROBE] {}",format_args!($($arg)*));
    }};
}

#[macro_export]
#[cfg(not(feature = "dbg-probe"))]
macro_rules! caravela_probe {
    ($($arg:tt)*) => {{}};
}

macro_rules! caravela_status {
    ($($arg:tt)*) => {{
        #[cfg(feature="dbg-status")]
        println!("[STATUS] {}",format_args!($($arg)*));
    }};
}
macro_rules! caravela_messaging {
    ($($arg:tt)*) => {{
        #[cfg(feature="dbg-messaging")]
        println!("[MESSAGING] {}",format_args!($($arg)*));
    }};
}
macro_rules! caravela_dflt {
    ($($arg:tt)*) => {{
        #[cfg(feature="dbg-default")]
        println!("[DEFAULT] {}",format_args!($($arg)*));
    }};
}

/// Macro to define agent types without parameters
#[macro_export]
macro_rules! make_agent {
    ($vis: vis $agent: ident) => {
        #[derive(Debug)]
        $vis struct $agent{
            agent: Agent
        }
        impl AgentBuild for $agent {
            fn agent_builder(agent: Agent) -> $agent {
                $agent{agent}
            }
        }

        impl AsMut<Agent> for $agent {
            fn as_mut(&mut self) -> &mut Agent {
                &mut self.agent
            }
        }
    };
}

/// Macro to define agent types with parameters
#[macro_export]
macro_rules! make_agent_with_param {
    ($vis: vis $agent: ident, $param_vis: vis $param_ty:ty) => {
        #[derive(Debug)]
        $vis struct $agent{
            agent: Agent,
            param: $param_ty
        }

        impl AgentBuildParam for $agent {
            type Parameter = $param_ty;
            fn agent_with_param_builder(agent: Agent, param: $param_ty) -> $agent {
                $agent{agent,param}
            }
            fn param(&mut self) -> &mut $param_ty{
                &mut self.1
            }
        }

        impl AsMut<Agent> for $agent {
            fn agent(&mut self) -> &mut Agent{
                &mut self.0
            }
        }
    };
}
