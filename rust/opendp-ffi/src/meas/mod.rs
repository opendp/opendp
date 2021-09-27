#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod gaussian;
#[cfg(feature="contrib")]
pub mod geometric;
#[cfg(feature="contrib")]
pub mod randomized_response;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod laplace;
#[cfg(all(feature="floating-point", feature="contrib"))]
pub mod stability;
