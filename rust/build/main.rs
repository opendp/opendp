#[cfg(not(feature = "derive"))]
fn main() {}

#[cfg(feature = "derive")]
mod derive;
#[cfg(feature = "derive")]
fn main() {
    crate::derive::main()
}
