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

#[macro_export]
macro_rules! agent{
    ($vis: vis $agent: ident) => {
        #[derive(Debug)]
        $vis struct $agent(Agent);
        impl AgentBuild for $agent {
            fn agent_builder(__0: Agent) -> $agent {
                $agent(__0)
            }
        }

        impl AgentBase for $agent {
            fn agent(&mut self) -> &mut Agent{
                &mut self.0
            }
        }
    };
}

//($vis: vis $agent: ident $(< $( $gen_lt:tt $( : $fbd:tt $(+ $sbd:tt )* )? ),+ >)? ) => {

#[macro_export]
macro_rules! agent_with_param {
    //($vis: vis $agent: ident, $pvis: vis $pty:ident $(< $( $glt:tt $(:$fbd:tt $(+ $sbd:tt )* )? ),+ >)? ) => {
    //todo!();
    //};
    ($vis: vis $agent: ident, $param_vis: vis $param_ty:ty) => {
        #[derive(Debug)]
        $vis struct $agent(Agent,$param_ty);

        impl AgentBuildParam for $agent {
            type Parameter = $param_ty;
            fn agent_with_param_builder(__0: Agent, __1: $param_ty) -> $agent {
                $agent(__0,__1)
            }
            fn param(&mut self) -> &mut $param_ty{
                &mut self.1
            }
        }

        impl AgentBase for $agent {
            fn agent(&mut self) -> &mut Agent{
                &mut self.0
            }
        }
    };
}
