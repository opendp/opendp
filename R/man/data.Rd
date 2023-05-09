% file testpkg/man/doFun.Rd
%
\name{convert}
\alias{slice_as_object}
\alias{object_as_slice}
\title{
Conversion of R objects to slices and viceversa
}
\description{
Conversion of R objects to slices and viceversa
}
\usage{
slice_as_object(data)
object_as_slice(obj)
}
\arguments{
  \item{data}{any R object}
  \item{obj}{an object of class AnyObject}
}
\value{
  The result is an object of clas AnyObject or an R object
}
\details{
 Types currently supported are 'i32', 'f64' and 'String' and their
 vector versions 'Vec<i32>', 'Vec<f64>' and 'Vec<String>'
}
\note{
 more types needed
}
\author{S. M. Iacus}
\examples{
require(opendp)
# create new data
x <- slice_as_object(10L) # integer 'i32'
x
str(object_as_slice(x))
y <- slice_as_object(c(1L,2L,3L))
str(object_as_slice(y))


x <- slice_as_object(100.) # float 'f64'
x
str(object_as_slice(x))
y <- slice_as_object(c(1,2,3))
str(object_as_slice(y))


x1 <- slice_as_object("A") # character 'String'
str(x1)
str(object_as_slice(x1))
x2 <- slice_as_object(LETTERS) # vector strings 'Vec<String>'
str(x2)
str(object_as_slice(x2))
x3 <- slice_as_object(c("Abc","Bcd","Cde")) # vector strings 'Vec<String>'
str(x3)
str(object_as_slice(x3))

rm(x)  # let's remove the object from memory 
rm(y)   
rm(x1)   
rm(x2)   
rm(x3)  
gc()
}
\keyword{data}

