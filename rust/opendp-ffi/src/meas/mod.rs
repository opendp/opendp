#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod gaussian;
#[cfg(feature="contrib")]
pub mod geometric;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod laplace;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod stability;
