% file testpkg/man/doFun.Rd
%
\name{applyFun}
\alias{applyFun}
\alias{squareIt}
\title{
Calling C code from package
}
\description{
  Calling C code from package
}
\usage{
squareIt(x)
applyFun(x, fun=NULL)
}
\arguments{
  \item{x}{a numeric for \code{squareIt} or any R object for \code{applyFun}}       
  \item{fun}{any R function}
}
\value{
  The result
}
\details{
  Test function to run C code or apply some R code from C.
}
\note{
 no sanity check may easily crash.
}
\author{S. M. Iacus}
\examples{
require(testpkg)
squareIt(4)
applyFun(pi/2,sin)
}
\keyword{data}

