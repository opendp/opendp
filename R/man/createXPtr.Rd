% file testpkg/man/doFun.Rd
%
\name{createXPtr}
\alias{createXPtr}
\alias{setXPtr}
\alias{getXPtr}
\title{
Creating and managing external pointers
}
\description{
Creating and managing external pointers
}
\usage{
createXPtr(info = "My XPtr obj")
setXPtr(x, str = "")
getXPtr(x)
}
\arguments{
  \item{x}{the external pointer}       
  \item{str}{a string}
  \item{info}{any R object, but we can use it to store the opendp library Type}
}
\value{
  The result is an External Pointer
}
\details{
  Test functions to create, set and get external pointer objects.
  Hard coded in C maximal string length equal to 15 chars.
}
\note{
 more sanity checks to be implemented
}
\author{S. M. Iacus}
\examples{
require(opendp)
x <- createXPtr("my info")
x # it's a list with a pointer to memory + other small things
str(x) # shows the structure
getXPtr(x) # return default initialized string
setXPtr(x, "my own string") # sets the content, returns TRUE
getXPtr(x)
x
rm(x)
gc()
# we removed the object from R workspace
# the finalizer will be called internally from C
# when the R garbage collector frees memory
}
\keyword{data}

