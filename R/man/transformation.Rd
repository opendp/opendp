% file testpkg/man/measurement.Rd
%
\name{invoke}
\alias{invoke}
\alias{map}
\alias{make_randomized_response}
\alias{make_base_gaussian}
\alias{make_base_discrete_laplace}
\title{
Measurements
}
\description{
Measurements
}
\usage{
invoke(this, data)
map(this, data)
make_clamp(bounds, TA="None")
make_bounded_sum(bounds, MI="SymmetricDistance", T="None") 
}
\arguments{
  \item{this}{Transformation object}
  \item{data}{Any R object}
  \item{MI}{has to be a string}
  \item{TA}{has to be a string}
  \item{T}{has to be a string}
}
\value{
  The result is an object of class Tranformation or an R object (for invoke or map)
}
\details{
 build and apply transformation to data.
}
\note{
 more sanity checks and object types to be implemented
}
\author{S. M. Iacus}
\examples{
require(opendp)
bounded_sum = make_bounded_sum(bounds=c(0., 5.))
bounded_mock_dataset = c(1.3, 3.8, 0., 5.)
invoke(bounded_sum, bounded_mock_dataset)
# 10.1
# same as
bounded_sum$invoke(bounded_mock_dataset)


clamp = make_clamp(bounds=c(0, 5))
invoke(clamp,c(1.3, 7.8, -2.5, 7.0))
# same as
clamp$invoke(c(1.3, 7.8, -2.5, 7.0))

# map
max_contributions = 2L
bounded_sum$map(max_contributions)

rm(bounded_sum)
rm(bounded_mock_dataset)
rm(clamp)
rm(max_contributions)
gc()
}
\keyword{data}

