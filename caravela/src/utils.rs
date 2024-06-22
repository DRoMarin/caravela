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
#[allow(dead_code)]
#[macro_export]
macro_rules! caravela_probe {
    ($($arg:tt)*) => {{
        #[cfg(feature="dbg-probe")]
        println!("[PROBE] {}",format_args!($($arg)*));
    }};
}
