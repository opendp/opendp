### file opendp/R/test.R



# creates an external pointer
createXPtr <- function(info = "My XPtr obj"){
  .Call("create",info, PACKAGE="opendp")
}

setXPtr <- function(x, str = ""){
  .Call("set", x$Ptr, str, PACKAGE="opendp")
}

getXPtr <- function(x){
  .Call("get",x$Ptr, PACKAGE="opendp")
}



# x : apply square function from C
squareIt <- function(x){
 .Call("square_It",x, PACKAGE="opendp")
}

# apply any function to x
applyFun <- function(x, fun = NULL){
  .Call("apply_Fun",x,fun, new.env(), PACKAGE="opendp")
}
