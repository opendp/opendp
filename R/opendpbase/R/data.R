### file opendp/R/data.R

hashmap <- function(keys, values){
  if(missing(keys) | missing(values))
    error("both keys and values must be provided")
  if(length(keys) != length(values))
    stop("keys and values mast have the same length")
  if( length(keys) != length(unique(keys)) )
    stop("keys must be unique")
  tmp <- new.env()
  tmp$keys <- keys
  tmp$values <- values
  tmp$get <- function(x) tmp$values[[x]]
  tmp$invoke <- function(x) {
    u <- as.list(match.call())
    ms <- as.character(u[[1]])[2]
    print(ms)
    ar <- u[[2]]
    print(ar)
    eval(call(ms,ar))
  }
  class(tmp) <- "hashmap"
  return(tmp)
}


print.hashmap <- function(x,...){
  print(data.frame(keys=x$keys, values = x$values))
  cat(sprintf("\n- class %s\n",class(x)))
}


rust_type <- function(x){
  len <- length(x)
  if(is.numeric(x) & !is.integer(x))
    return(ifelse(len>1, "Vec<f64>", "f64"))
  
  if(is.integer(x))
    return(ifelse(len>1, "Vec<i32>", "i32"))
  
  if(is.character(x))
    return( ifelse(len>1, "Vec<String>", "String") )
  
  if(is.list(x)){
    rtypes <- NULL
    for(i in 1:len){
      rtypes <- c(rtypes, rust_type(x[[i]]))
    }
    return(rtypes)
  }
}

resolve_type <- function(x){
  lst <- NULL
  len <- length(x)
  if(is.numeric(x)){
    type_name <- ifelse(len>1, "Vec<f64>", "f64")
    c_type <- "double"
    lst <- c(type_name, c_type)
  }
  
  if(is.integer(x)){
    type_name <- ifelse(len>1, "Vec<i32>", "i32")
    c_type <- "int"
    lst <- c(type_name, c_type)
  }
  
  if(is.character(x)){
    type_name <- ifelse(len>1, "Vec<String>", "String")
    c_type <- "char"
    lst <- c(type_name, c_type)
  }
  
  if(is.list(x)){
    for(i in 1:len){
      lst <- rbind(lst, resolve_type(x[[i]]))
    }
  }
  if(is.null(lst)) return(NULL)
  
  if(is.array(lst)){
    colnames(lst) <- c("type_name", "c_type")
  } else {
    names(lst) <- c("type_name", "c_type")
  }  
  return(lst)
}

R_to_C <- function(value, c_type=NULL, type_name=NULL){
  # len <- length(value)
  # if(is.numeric(value)){
  #   type_name <- ifelse(len>1, "Vec<f64>", "f64")
  #   if(is.null(c_type))
  #     c_type <- "double"
  # }
  # 
  # if(is.integer(value)){
  #   type_name <- ifelse(len>1, "Vec<i32>", "i32")
  #   if(is.null(c_type))
  #     c_type <- "int"
  # }
  # 
  # if(is.character(value)){
  #   type_name <- ifelse(len>1, "Vec<String>", "String")
  #   if(is.null(c_type))
  #     c_type <- "char"
  # }
  print(c_type)
  if(!is.null(c_type)){
    if(c_type=="AnyObjectPtr"){
      retval <- slice_as_object(value)
      return(retval)
    } 
  }
  
  if(is.list(value)) {
    tmp <- resolve_type(value)
    type_name <- tmp[,"type_name"]
    c_type <- tmp[,"c_type"]
    retval <- .Call("odp_R_to_C", value, c_type, type_name, PACKAGE = 'opendpbase')
    # class(retval) <- c(class(retval))
    # attr(retval, "type") <- tmp
    return(retval)
  }
  
  tmp <- resolve_type(value)
  
  if(is.null(tmp)) return(NULL)
  
  if(is.array(tmp)){
    type_name <- tmp[,"type_name"]
    c_type <- tmp[,"c_type"]
    retval <- .Call("odp_R_to_C", value, c_type, type_name, PACKAGE = 'opendpbase')
    class(retval) <- c(class(retval))
    attr(retval, "type") <- tmp
  } else {
    type_name <- tmp["type_name"]
    c_type <- tmp["c_type"]
  }
  retval <- .Call("odp_R_to_C", value, c_type, type_name, PACKAGE = 'opendpbase')
  class(retval) <- c(class(retval))
  attr(retval, "type") <- tmp
  return(retval)
}

object_as_slice <- function(obj){
  if(!inherits(obj, "AnyObject")){
    stop("Not an object of class 'AnyObject'")
  }
  retval <- .Call("odp_object_as_slice", obj$AnyObjectPtr, PACKAGE = 'opendpbase')
  return(retval)
}

slice_as_object <- function(data) {
  retval <- .Call("odp_slice_as_object", data, PACKAGE = 'opendpbase')
  class(retval) <- "AnyObject"
  return(retval)
}


print.AnyObject <- function(x,...){
  cat(sprintf("OpenDP 'AnyObject'"))
  cat(sprintf("\n- type: %s",x$type))
  cat(sprintf("\n- length: %d",x$length))
  cat(sprintf("\n- %s\n",capture.output(x$AnyObjectPtr)))
}



