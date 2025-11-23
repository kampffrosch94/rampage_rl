#[cfg(feature = "profile")]
macro_rules! zone {
    ($($arg:tt)*) => {
        tracy::zone!($($arg)*);
    };
}

#[cfg(not(feature = "profile"))]
macro_rules! zone {
    ($($arg:tt)*) => {};
}
