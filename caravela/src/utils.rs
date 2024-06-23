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
