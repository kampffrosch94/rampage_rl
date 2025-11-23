#[cfg(feature = "profile")]
#[macro_export]
macro_rules! zone {
    ($($arg:tt)*) => {
        ::base::tracy::zone!($($arg)*);
    };
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! frame {
    ($($arg:tt)*) => {
        ::base::tracy::frame!($($arg)*);
    };
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! zone {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! frame {
    ($($arg:tt)*) => {};
}
