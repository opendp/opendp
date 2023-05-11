% file testpkg/man/comb.Rd
%
\name{make_chain_tt}
\alias{make_chain_mt}
\title{
Chaining measurements and transformations
}
\description{
Chaining measurements and transformations
}
\usage{
make_chain_mt(meas, tran)
make_chain_tm(tran,meas)
make_chain_tt(tran1,tran2)
make_pureDP_to_fixed_approxDP(meas)
make_zCDP_to_approxDP(meas)
make_fix_delta <- function(meas, delta)
}
\arguments{
  \item{meas}{Measurement object}
  \item{tran}{Measurement object}
  \item{tran1}{Measurement object}
  \item{tran2}{Measurement object}
  \item{delta}{a real number}
}
\value{
  The result is an object of class Measurement
}
\details{
 chaining measurements and transformations
}
\note{
 more sanity checks and object types to be implemented
}
\author{S. M. Iacus}
\examples{
require(opendpbase)

# chaining
aggregator = make_bounded_sum(bounds=c(0L, 10L))
mechanism = make_base_discrete_laplace(scale=1)
meas = make_chain_mt(mechanism, aggregator)
invoke(meas, c(1L, 2L, 3L, 7L))

bounds <- c(0, 5)
meas = make_chain_mt(make_base_laplace(5.), make_bounded_sum(bounds))
data <- c(1,3,5,7,9)
meas$invoke(data)
tran = make_chain_tt(make_bounded_sum(bounds),make_clamp(bounds))
tran$invoke(data)

comb1 <- make_pureDP_to_fixed_approxDP(make_base_laplace(10.))
comb2 <- make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6)
comb1$invoke(10)
comb2$invoke(10)

m1 <- make_pureDP_to_fixed_approxDP(make_base_laplace(10.))
m2 <- make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6)
bcomp <- make_basic_composition(m1,m2)

m1$invoke(10)
m2$invoke(10)
bcomp$invoke(10)
bcomp$map(1)
print(bcomp$map(1), digits=16)

bounded_mock_dataset <- c(1.3, 3.8, 0., 5.)
bounded_sum <- make_bounded_sum(bounds=c(0., 5.))
bounded_sum$invoke(bounded_mock_dataset)
meas <- make_basic_composition(
        make_pureDP_to_fixed_approxDP(make_base_laplace(10.)),
        make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6))

print(meas$map(1.), digits=16)

max_contributions <- 2L
mp <- bounded_sum$map(max_contributions)
print(mp, digits=16)

clamp <- make_clamp(bounds=c(0., 5.))
mock_dataset = c(1.3, 7.8, -2.5, 7.0)
clamp$invoke(mock_dataset)
clamp$map(max_contributions)

# `%>%.measurement` <- function(lh, rh) {make_chain_mt(rh, lh)}

trans1 = make_chain_tt(make_bounded_sum(bounds), make_clamp(bounds))
meas = make_chain_mt(make_base_laplace(5.), trans1)
meas

# working
curve <- make_zCDP_to_approxDP(make_base_gaussian(4.))
curve
cv <- curve$map(1.)
cv$epsilon(1e-3)
delta <- seq(1e-5,1e-3, length=100)
eps <- sapply(delta, function(x) cv$epsilon(x))
plot(delta, eps, type="l")


max_contributions = 2L
bounds = c(0., 5.)

# construct the measurement
# dp_sum = make_clamp(bounds) >> make_bounded_sum(bounds) >> make_base_laplace(5.)
chain1 = make_chain_tt(make_bounded_sum(bounds),make_clamp(bounds))
dp_sum = make_chain_mt(make_base_laplace(5.),chain1)

chain1$map(2L)
dp_sum$map(2L)

# evaluate the privacy expenditure and make a DP release
mock_dataset = c(0.7, -0.3, 1., -1.)
cat(sprintf("\n epsilon: %f\n", dp_sum$map(max_contributions)))
cat(sprintf("\n DP sum release: %f\n", dp_sum$invoke(mock_dataset)))

dp_sum2 = make_chain_tt(make_bounded_sum(bounds),make_clamp(bounds))
dp_sum2 = make_chain_mt(make_base_laplace(5.),dp_sum2)

# evaluate the privacy expenditure and make a DP release
mock_dataset = c(0.7, -0.3, 1., -1.)
cat(sprintf("\n epsilon: %f\n", dp_sum$map(max_contributions)))
cat(sprintf("\n DP sum release: %f\n", dp_sum$invoke(mock_dataset)))

#def test_amplification():
#    from opendp.transformations import make_sized_bounded_mean
#
#    meas = make_sized_bounded_mean(size=10, bounds=(0., 10.)) >> make_base_laplace(scale=0.5)
#    meas = make_chain_mt(make_base_laplace(scale=0.5), make_sized_bounded_mean(size=10, bounds=(0., 10.)))
#    amplified = make_population_amplification(meas, population_size=100)
#    print("amplified base laplace:", amplified([1.] * 10))
#    assert meas.check(2, 2. + 1e-6)
#    assert not meas.check(2, 2.)
#    assert amplified.check(2, 1.494)
#    assert not amplified.check(2, .494)
    
trans1 <- make_sized_bounded_mean(size=10, bounds=c(0., 10.))
meas <- make_chain_mt(make_base_laplace(scale=0.5), trans1) 
amplified <- make_population_amplification(meas, population_size=100)
cat("\namplified base laplace:")
invoke(amplified, rep(1.,  10))
check(meas, 2L, 0.) # should and does return FALSE
check(meas, 2L, 2. + 1e-6) # should be TRUE
check(meas, 2L, 2.) # should be FALSE, returns FALSE

check(amplified, 2L, 1.494) # should return TRUE
check(amplified, 2L, .494) # should return FALSE, returns FALSE
check(meas, 2L, .Machine$double.xmin) # should return FALSE

#def test_fix_delta():
#    base_gaussian = make_zCDP_to_approxDP(make_base_gaussian(10.))
#    print(base_gaussian.map(1.).epsilon(1e-6))
##   0.42994146883694945
#    fixed_base_gaussian = make_fix_delta(base_gaussian, 1e-6)
#
#    print(fixed_base_gaussian.map(1.))
##   (0.42994146883694945, 1e-06)  

base_gaussian <-  make_zCDP_to_approxDP(make_base_gaussian(10.))
base_gaussian$map(1.)$epsilon(1e-6) # 0.42994146883694945
fixed_base_gaussian <-  make_fix_delta(base_gaussian, 1e-6)
fixed_base_gaussian$map(1.) # (0.42994146883694945, 1e-06)

#def test_make_basic_composition():
#    composed = make_basic_composition([
#        make_count(TIA=int, TO=int) >> make_basic_composition([
#            make_base_discrete_laplace(scale=2.), 
#            make_base_discrete_laplace(scale=200.)
#        ]), 
#        make_cast_default(int, bool) >> make_cast_default(bool, int) >> make_count(TIA=int, TO=int) >> make_base_discrete_laplace(scale=2.), 
#        make_cast_default(int, float) >> make_clamp((0., 10.)) >> make_bounded_sum((0., 10.)) >> make_base_laplace(scale=2.), 
#
#        make_basic_composition([
#            make_count(TIA=int, TO=int) >> make_base_discrete_laplace(scale=2.), 
#            make_count(TIA=int, TO=float) >> make_base_laplace(scale=2.),
#            (
#                make_cast_default(int, str) >> 
#                make_count_by_categories(categories=["0", "12", "22"]) >> 
#                make_base_discrete_laplace(scale=2., D=VectorDomain[AtomDomain[int]])
#            )
#        ])
#    ])
 
# chain1   
# make_count(TIA=int, TO=int) >> make_basic_composition([
#            make_base_discrete_laplace(scale=2.), 
#            make_base_discrete_laplace(scale=200.)
#        ])   
tra <- make_count(TIA="int", TO="int")
comp <- make_basic_composition(make_base_discrete_laplace(scale=2.), 
            make_base_discrete_laplace(scale=200.))   
chain1 <- make_chain_mt(comp, tra)

# chain2
#  make_cast_default(int, bool) >> make_cast_default(bool, int) >> make_count(TIA=int, TO=int) >> make_base_discrete_laplace(scale=2.)
ch1 <- make_chain_tt(make_cast_default("bool", "int"), make_cast_default("int", "bool"))
ch2 <- make_chain_tt(make_count(TIA="int", TO="int"), ch1)
chain2 <- make_chain_mt(make_base_discrete_laplace(scale=2.), ch2)

# chain3
# make_cast_default(int, float) >> make_clamp((0., 10.)) >> make_bounded_sum((0., 10.)) >> make_base_laplace(scale=2.)
ch4 <- make_chain_tt(make_clamp(c(0., 10.)), make_cast_default("int", "float"))
ch5 <- make_chain_tt(make_bounded_sum(c(0., 10.)), ch4)
chain3 <- make_chain_mt(make_base_laplace(scale=2.), ch5)

#   make_count(TIA=int, TO=int) >> make_base_discrete_laplace(scale=2.), 
#            make_count(TIA=int, TO=float) >> make_base_laplace(scale=2.),
#        

ch7 <- make_chain_mt( make_base_discrete_laplace(scale=2.), make_count(TIA="int", TO="int"))
ch8 <- make_chain_mt(  make_base_laplace(scale=2.), make_count(TIA="int", TO="float"))

# make_cast_default(int, str) >> 
#                make_count_by_categories(categories=["0", "12", "22"]) >> 
#                make_base_discrete_laplace(scale=2., D=VectorDomain[AtomDomain[int]])


ch9 <- make_chain_tt(make_count_by_categories(categories=c("0", "12", "22")), make_cast_default("int", "str"))
chA <-  make_chain_mt(make_base_discrete_laplace(scale=2., D="VectorDomain[AtomDomain[int]]"), ch9)

bc3 <- make_basic_composition(ch7,ch8, chA)

composed <- make_basic_composition(chain1, chain2, chain3, bc3)

cat("\nCheck:")
check(composed, 1L, 2.)
# >>> print("Check:", composed.check(1, 2.))
# Check: False

cat("\nForward map:")
composed$map(3L)
# >>> print("Forward map:", composed.map(3))
# Forward map: 22.515000046566133
print(composed$map(3L),digits=17)

cat("Invocation:")
composed$invoke(c(22L, 12L))


#def test_make_basic_composition_approx():
#    composed_fixed = make_basic_composition([
#        make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(1.)), 1e-7)
#    ] * 2)
#    print(composed_fixed.map(1.))

composed_fixed = make_basic_composition(
        make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(1.)), 1e-7),
         make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(1.)), 1e-7))
composed_fixed$map(1.)  # (11.341717670937543, 2e-07)

#def test_make_pureDP_to_fixed_approxDP():
#    meas = make_basic_composition([
#        make_pureDP_to_fixed_approxDP(make_base_laplace(10.)),
#        make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6)
#    ])
#
#    print(meas.map(1.))

meas <- make_basic_composition(
        make_pureDP_to_fixed_approxDP(make_base_laplace(10.)),
        make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6))
meas$map(1.) # (0.5299414688369495, 1e-06)

#def test_make_pureDP_to_zCDP():
#    meas = make_basic_composition([
#        make_pureDP_to_zCDP(make_base_laplace(10.)),
#        make_base_gaussian(10.)
#    ])
#
#    print(meas.map(1.))
# 0.010000000000000002

meas <- make_basic_composition(make_pureDP_to_zCDP(make_base_laplace(10.)),
        make_base_gaussian(10.))
print(meas$map(1.),digits=20) #0.010000000000000002

rm(bcomp)
rm(bounded_mock_dataset)
rm(bounded_sum)
rm(clamp)
rm(cv)
rm(m1)
rm(m2)
rm(max_contributions)
rm(mock_dataset)
rm(mp)
rm(curve)
rm(delta)
rm(eps)
rm(aggregator)
rm(mechanism)
rm(meas)
rm(tran)
rm(trans1)
rm(data)
rm(comb1)
rm(comb2)
rm(bounds)
gc()
}
\keyword{data}

