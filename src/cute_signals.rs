/// Emulates Qt's signal/slot connection mechanism for Slint.
/// Provides a clean macro to connect a Slint UI signal to a Rust closure.
#[macro_export]
macro_rules! cute_connect {
    ($sender:expr, $signal:ident, $slot:expr) => {
        $sender.$signal($slot);
    };
}
