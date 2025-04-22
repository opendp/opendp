#[cfg(feature = "derive")]
mod derive;

fn main() {
    #[cfg(feature = "derive")]
    crate::derive::main();
}
