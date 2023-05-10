### file opendp/R/comb.R


# meas = make_basic_composition([
#   make_pureDP_to_fixed_approxDP(make_base_laplace(10.)),
#   make_fix_delta(make_zCDP_to_approxDP(make_base_gaussian(10.)), delta=1e-6)
# ])
# 
# print(meas.map(1.))

make_basic_composition <- function(...){
  m <- list(...)
  n <- length(m)
  # for(i in 1:n) {
  #   print(str(m[[i]]))
  # }
  measVec <- vector("list", n)
  for(i in 1:n) {
    ma <- m[[i]]
    measVec[[i]] <-  ma$MeasurementPtr
  }
  class(measVec) <- "measVec"
  # print(str(v))
  # return(v)
  info <- list(name="make_basic_composition", measVec=measVec)
  
  retval <- .Call("odp_make_basic_composition", measVec, info, PACKAGE = 'opendp')
  returnMeas(retval)
}


make_chain_mt <- function(meas, tran) {
  info <- list(name="make_chain_mt", meas=getInfo(meas)$parameters, tran=getInfo(tran)$parameters)
  retval <- .Call("odp_make_chain_mt", meas$MeasurementPtr, tran$TransformationPtr, info, PACKAGE = 'opendp')
  returnMeas(retval)
}


make_chain_tm <- function(tran, meas) {
  info <- list(name="make_chain_tm", tran=getInfo(tran), meas=getInfo(meas))
  retval <- .Call("odp_make_chain_tm",  tran$TransformationPtr, meas$MeasurementPtr, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

make_chain_tt <- function(tran1, tran2) {
  info <- list(name="make_chain_tt", tran1=getInfo(tran1), tran2=getInfo(tran2))
  retval <- .Call("odp_make_chain_tt",  tran1$TransformationPtr, tran2$TransformationPtr, info, PACKAGE = 'opendp')
  returnTran(retval)
}


make_population_amplification <- function(meas, population_size) {
  population_size <- as.integer(population_size)
  info <- list(name="make_population_amplification", meas = meas, population_size = population_size)
  retval <- .Call("odp_make_population_amplification", meas$MeasurementPtr, population_size, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

make_pureDP_to_fixed_approxDP <- function(meas) {
  info <- list(name="make_pureDP_to_fixed_approxDP", meas=meas)
  retval <- .Call("odp_make_pureDP_to_fixed_approxDP",  meas$MeasurementPtr, info, PACKAGE = 'opendp')
  returnMeas(retval)
}


make_pureDP_to_zCDP <- function(meas) {
  info <- list(name="make_pureDP_to_zCDP", meas=meas)
  retval <- .Call("odp_make_pureDP_to_zCDP",  meas$MeasurementPtr, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

make_zCDP_to_approxDP <- function(meas) {
  info <- list(name="make_zCDP_to_approxDP", meas=meas)
  retval <- .Call("odp_make_zCDP_to_approxDP",  meas$MeasurementPtr, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

make_fix_delta <- function(meas, delta) {
  info <- list(name="make_fix_delta", meas=meas, delta=delta)
  retval <- .Call("odp_make_fix_delta",  meas$MeasurementPtr, delta, info, PACKAGE = 'opendp')
  returnMeas(retval)
}



