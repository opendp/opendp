% file testpkg/man/comb.Rd
%
\name{binary_search_chain}
\alias{binary_search_chain}
\title{
Binary search chain
}
\description{
Binary search chain
}
\usage{
binary_search_chain(make_chain, d_in, d_out, bounds=NULL, T=NULL)
}
\arguments{
  \item{make_chain}{the chain measurement}
  \item{d_in}{domain input}
  \item{d_out}{domain output}
  \item{bounds}{\code{NULL} or vector}
  \item{T}{\code{NULL} or type}
}
\value{
  The result is an object of class Measurement
}
\details{
 Binary search chain searches for the optimal parameter
}
\author{S. M. Iacus}
\examples{
library(opendpbase)
pre <- make_bounded_sum(bounds=c(0., 1.))
fun <- function(s) make_chain_mt(make_base_laplace(scale=s), pre)

chain1 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1., bounds=c(0.,100.) , T="float")
chain1

chain2 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1. , T="float")
chain2

chain3 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1. )
chain3

pre <- make_base_discrete_laplace(scale=1.)
fun <- function(s) make_chain_mt(pre, make_bounded_sum(bounds=c(0L, s)))

chain1 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1., bounds=c(0L,100L) , T="int")
chain1

chain2 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1. , T="int")
chain2

chain3 = binary_search_chain(function(x) fun(x), d_in=1L, d_out=1. )
chain3
rm(chain1)
rm(chain2)
rm(chain3)
rm(pre)
rm(fun)
gc()
}
\keyword{data}

