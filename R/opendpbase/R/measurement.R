### file opendp/R/measurement.R



getInfo <- function(this){
  retval <- NULL
  if(inherits(this, "Measurement")){
    retval <- .Call("odp_getMeasurementInfo", this$MeasurementPtr, PACKAGE = 'opendp')
  }
   if(inherits(this, "Transformation")){
     retval <- .Call("odp_getTransformationInfo", this$TransformationPtr, PACKAGE = 'opendp')
  }
  
  retval
}


returnSMDCurve <- function(curve){
  curve$epsilon <- function(delta) {
    smd_curve_epsilon(curve = curve$SMDCurvePtr, delta = delta)
  }
  return(curve)
}


smd_curve_epsilon <- function(curve, delta){
  retval <- .Call("odp_smd_curve_epsilon", curve, delta, PACKAGE = 'opendp')
  retval
}

map <- function(this, data){
  retval <- NULL
  if(inherits(this, "Measurement")){
    retval <- .Call("odp_measurement_map", this$MeasurementPtr, data, PACKAGE = 'opendp')
  }
  if(inherits(this, "Transformation")){
    retval <- .Call("odp_transformation_map", this$TransformationPtr, data, PACKAGE = 'opendp')
  }
  if(inherits(retval,"SMDCurve")){
    returnSMDCurve(retval)
  } else {
   return(retval)
  }
}
  
invoke <- function(this, data){
  retval <- NULL
  if(inherits(this, "Measurement")){
    retval <- .Call("odp_measurement_invoke", this$MeasurementPtr, data, PACKAGE = 'opendp')
  }
  if(inherits(this, "Transformation")){
    retval <- .Call("odp_transformation_invoke", this$TransformationPtr, data, PACKAGE = 'opendp')
  }
  
  retval
}




returnMeas <- function(m){
  class(m) <- "Measurement"
  m$invoke <- function(x) {
    u <- as.list(match.call())
    ms <- as.character(u[[1]])[2]
    invoke(get(ms),data=x)
  }
  m$map <- function(x) {
    u <- as.list(match.call())
    ms <- as.character(u[[1]])[2]
    map(get(ms),data=x)
  }
  return(m)
}


make_base_ptr <- function(scale, threshold, TK, k= -1074L, TV) {
    info <- list(name="make_base_ptr", scale = scale, threshold = threshold, TK = TK, k=k, TV=TV)
    retval <- .Call("odp_make_base_ptr", scale, threshold, TK, k, TV, info, PACKAGE = 'opendp')
    returnMeas(retval)
}





make_randomized_response <- function(categories, prob, constant_time = FALSE) {
  T <- rust_type(categories[1])
  QO <- rust_type(prob)
  info <- list(name="make_randomized_response", categories = categories, prob = prob, constant_time = constant_time, T=T, QO=QO)
  retval <- .Call("odp_make_randomized_response", categories, prob, constant_time, T, QO, info, PACKAGE = 'opendp')
  returnMeas(retval)
}


make_base_gaussian <- function(scale, k=-1074L, D="AllDomain", MO="ZeroConcentratedDivergence") {
  T <- rust_type(scale)
  D <- sprintf("%s<%s>",D,T)
  MO <- sprintf("%s<%s>",MO,T)
  info <- list(name="make_base_gaussian", scale = scale, k = k, D = D, MO=MO)
  retval <- as.environment(.Call("odp_make_base_gaussian", scale, k, D, MO, info, PACKAGE = 'opendp'))
  returnMeas(retval)
}

make_base_laplace <- function(scale, k=-1074L, D="AllDomain") {
  T <- rust_type(scale)
  D <- sprintf("%s<%s>",D,T)
  info <- list(name="make_base_laplace", scale = scale, k = k, D = D)
  retval <- .Call("odp_make_base_laplace", scale, k, D, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

nodmalizeDomain <- function(str){
  str <- stringr::str_replace_all(str,"\\[","<")
  str <- stringr::str_replace_all(str,"\\]",">")
  str <- stringr::str_replace_all(str,"int","i32")
  str <- stringr::str_replace_all(str,"float","f64")
  str <- stringr::str_replace_all(str,"str","String")
}

make_base_discrete_laplace <- function(scale, D="AllDomain<int>", QO="None") {
  QO <- rust_type(scale)
  D <- nodmalizeDomain(D)
#  D <- sprintf("%s<%s>",D,rust_type(1L))
  info <- list(name="make_base_discrete_laplace", scale = scale, D = D, QO = QO)
  retval <- .Call("odp_make_base_discrete_laplace", scale, D, QO, info, PACKAGE = 'opendp')
  returnMeas(retval)
}

print.Measurement <- function(x,...){
  cat(sprintf("OpenDP 'Measurement'"))
  cat("\n\nparameters:")
  params <- getInfo(x)$parameters
  n <- length(params)
  for(i in 1:n){
    try(cat(sprintf("\n- %s: %s",names(params[i]), as.character(params[[i]]))), TRUE)
  }
  cat("\n\nrust types:")
  otherparams <- getInfo(x)
  n <- length(otherparams)-1
  for(i in 1:n){
    try(cat(sprintf("\n- %s: %s",names(otherparams[i]), as.character(otherparams[[i]]))), TRUE)
  }
  
  cat(sprintf("\n\n%s\n",capture.output(x$MeasurementPtr)))
}

