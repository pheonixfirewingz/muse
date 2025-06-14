#[macro_export]
macro_rules! debug_println {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        #[cfg(debug_assertions)]
        {
            println!(concat!("debug: ", $fmt) $(, $arg)*);
        }
    };
}
