
binary_search_chain <- function(make_chain, d_in, d_out, bounds=NULL, T=NULL){
  return ( make_chain(binary_search_param(make_chain, d_in, d_out, bounds, T)) )
}

binary_search_param <- function(make_chain, d_in, d_out, bounds, T){
  return (binary_search(function(param){ check(make_chain(param), d_in, d_out) }, bounds, T) )
}

binary_search <- function( predicate, bounds = NULL,T=NULL, return_sign=FALSE){
 
  if(is.null(bounds)){
    bounds = exponential_bounds_search(predicate, T)
  }
  
  if(is.null(bounds)){
    stop("unable to infer bounds")
  }
  
  # if(len(set(map(type, bounds))) != 1){
  #   stop("bounds must share the same type")
  # }
  
 # print(str(bounds))
   tmp <- sort(bounds)
   lower <- tmp[1]
   upper <- tmp[2] 

  # print("here")
  # print(str(c(lower, upper)))
#  print("doing maximize")
  maximize = predicate(lower)  # if the lower bound passes, we should maximize
#  print(str(maximize))
#  print("doing minimize")
  minimize = predicate(upper)  # if the upper bound passes, we should minimize
#  print(str(minimize))
#  print(c(maximize, minimize))
  if(maximize == minimize){
    stop("the decision boundary of the predicate is outside the bounds")
  }

  if(inherits(lower,"numeric")){
    tolerance = 0.
    half = function(x) return(x / 2.)
  } else {
    if(inherits(lower,"integer")){
      tolerance = 1L  # the lower and upper bounds never meet due to int truncation
      half = function(x) return(x %/% 2L)
    } else {
      stop("bounds must be either float or int")
    }
  }

  mid = lower
  while(upper - lower > tolerance){
    new_mid = lower + half(upper - lower)  # avoid overflow
   #cat(sprintf("\n lower=%f, upper=%f, tolerance=%f, new_mid=%f", lower, upper, tolerance, new_mid))
    # avoid an infinite loop from float roundoff
    if(new_mid == mid) break;
    mid = new_mid
  
    
    if(predicate(mid) == minimize){
      upper = mid
    } else {
      lower = mid
    }
  }
# one bound is always false, the other true. Return the truthy bound
  value = ifelse(minimize, upper ,lower)

# optionally return sign
  if(return_sign){
    return( c(value, ifelse(minimze,  1 , -1)) )
  }
  return (value)
}

exponential_bounds_search <- function(predicate, T){

  # try to infer T
  if(is.null(T)){
    check_type <- function(v) {
      f <- try(predicate(v), TRUE)
      if(inherits(f,"try-error")){
        #warning(attr(f,"condition")$message)
        return(FALSE)
      } else {
        return(TRUE)
      }
    }
  

    if(check_type(0.)){
      T = "float"
    } else {
        if(check_type(0L)){
          T = "int"
        } else {
          stop("unable to infer type `T`; pass the type `T` or bounds")
        }
      }
  }

# core search functionality
  signed_band_search <- function(center, at_center, sign){
    if(T == "int"){
      bands = as.integer(c(center, center + 1, (center + sign * 2 ** 16 * (0:(9-1)) )))
    }
    if(T == "float"){
      bands = c(center, (center + sign * 2. ** (0:(1024 %/% 32 %/% 4 -1) ) ** 2) )
    }

    for(i in 2:(length(bands)-1)){
  #   looking for a change in sign that indicates the decision boundary is within this band
      if(at_center != predicate(bands[i])){
  # return the band
        return( c(sort(bands[(i - 1):i ])) )
      }
    }
# No band found!
    return(NULL)
  }
  
  if(T=="int") center = 0L
  if(T=="float") center = 0.
  
  at_center = try(predicate(center), TRUE)
# search positive bands, then negative bands
  ret <- try(signed_band_search(center, at_center, 1), TRUE)
 
  if(is.null(ret) ) {
    ret <- try(signed_band_search(center, at_center, -1), TRUE)
  }
 # print(str(ret))
  if(!inherits(at_center,"try-error") & !inherits(ret,"try-error")){
    return(ret)
  }

# predicate has thrown an exception
# 1. Treat exceptions as a secondary decision boundary, and find the edge value
# 2. Return a bound by searching from the exception edge, in the direction away from the exception
  exception_predicate <- function(v){
    f <- try(predicate(v), TRUE)
    if(inherits(f,"try-error")){
      return(FALSE)
    } else {
      return(TRUE)
    }
  }
    
  exception_bounds <- exponential_bounds_search(exception_predicate, T=T)
  
  print( exception_bounds )

    
  if(is.null(exception_bounds)){  # not sure about this code
    f1 <- try(predicate(center), TRUE)
    if(inherits(f1,"try-error")){
      err = f1$message
      error = sprintf(". Error at center: %s", err)
      stop(sprintf("predicate always fails%s", err))
    }
  }
  
  tmp <- binary_search(exception_predicate, bounds=exception_bounds, T=T, return_sign=True)
  center <- tmp[1]
  if(length(tmp)>1){
    sign <- tmp[2]
  }
  at_center = predicate(center)
  return(signed_band_search(center, at_center, sign))
}



# [docs]def exponential_bounds_search(
#   predicate: Callable[[Union[float, int]], bool], 
#   T: Optional[type]) -> Optional[Union[Tuple[float, float], Tuple[int, int]]]:
#   """Determine bounds for a binary search via an exponential search,
#     in large bands of [2^((k - 1)^2), 2^(k^2)] for k in [0, 8).
#     Will attempt to recover once if `predicate` throws an exception, 
#     by searching bands on the ok side of the exception boundary.
#     
# 
#     :param predicate: a monotonic unary function from a number to a boolean
#     :param T: type of argument to predicate, one of {float, int}
#     :return: a tuple of float or int bounds that the decision boundary lies within
#     :raises TypeError: if the type is not inferrable (pass T)
#     :raises ValueError: if the predicate function is constant
#     """
# 
# # try to infer T
# if T is None:
#   def check_type(v):
#   try:
#   predicate(v)
# except TypeError as e:
#   return False
# except OpenDPException as e:
#   if "No match for concrete type" in e.message:
#   return False
# return True
# 
# if check_type(0.):
#   T = float
# elif check_type(0):
#   T = int
# else:
#   raise TypeError("unable to infer type `T`; pass the type `T` or bounds")
# 
# # core search functionality


# def signed_band_search(center, at_center, sign):
#   """identify which band (of eight) the decision boundary lies in,
#         starting from `center` in the direction indicated by `sign`"""
# 
# if T == int:
#   # searching bands of [(k - 1) * 2^16, k * 2^16].
#   # center + 1 included because zero is prone to error
#   bands = [center, center + 1, *(center + sign * 2 ** 16 * k for k in range(1, 9))]
# 
# if T == float:
#   # searching bands of [2^((k - 1)^2), 2^(k^2)].
#   # exponent has ten bits (2.^1024 overflows) so k must be in [0, 32).
#   # unlikely to need numbers greater than 2**64, and to avoid overflow from shifted centers,
#   #    only check k in [0, 8). Set your own bounds if this is not sufficient
#   bands = [center, *(center + sign * 2. ** k ** 2 for k in range(1024 // 32 // 4))]
# 
# for i in range(1, len(bands)):
#   # looking for a change in sign that indicates the decision boundary is within this band
#   if at_center != predicate(bands[i]):
#   # return the band
#   return tuple(sorted(bands[i - 1:i + 1]))
# 
# # No band found!
# return None
# 
# try:
#   center = {int: 0, float: 0.}[T]
# at_center = predicate(center)
# # search positive bands, then negative bands
# return signed_band_search(center, at_center, 1) or signed_band_search(center, at_center, -1)
# except:
#   pass
# 
# # predicate has thrown an exception
# # 1. Treat exceptions as a secondary decision boundary, and find the edge value
# # 2. Return a bound by searching from the exception edge, in the direction away from the exception
# def exception_predicate(v):
#   try:
#   predicate(v)
# return True
# except:
#   return False
# exception_bounds = exponential_bounds_search(exception_predicate, T=T)
# if exception_bounds is None:
#   error = ""
# try:
#   predicate(center)
# except Exception as err:
#   error = f". Error at center: {err}"
# 
# raise ValueError(f"predicate always fails{error}")
# 
# center, sign = binary_search(exception_predicate, bounds=exception_bounds, T=T, return_sign=True)
# at_center = predicate(center)
# return signed_band_search(center, at_center, sign)
# 










