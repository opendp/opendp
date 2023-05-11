### file opendp/R/transformation.R



returnTran <- function(t){
  class(t) <- "Transformation"
  t$invoke <- function(x) {
    u <- as.list(match.call())
    ms <- as.character(u[[1]])[2]
    invoke(get(ms),data=x)
  }
  t$map <- function(x) {
    u <- as.list(match.call())
    ms <- as.character(u[[1]])[2]
    map(get(ms),data=x)
  }
  return(t)
}


make_tuple <- function(x){
  return(as.list(x))
}

check <- function(this, distance_in, distance_out){
  # distance_in <- getInfo(this)$input_distance
  # distance_out <- getInfo(this)$output_distance
  retval <- FALSE
  if(inherits(this,"Measurement"))
    retval <- .Call("odp_measurement_check", this$MeasurementPtr, distance_in, distance_out, PACKAGE = 'opendpbase')
  if(inherits(this,"Transformation"))
    retval <- .Call("odp_transformation_check", this$TransformationPtr, distance_in, distance_out, PACKAGE = 'opendpbase')
  return(retval)
}


toRust <- function(x){
  switch (x,
    "int" = "i32",
    "bool" = "bool",
    "float" = "f64",
    "str" = "String"
  )
}

make_cast_default <- function(TIA, TOA){
  TIA <- toRust(as.character(TIA))
  TOA <- toRust(as.character(TOA))
  
  info <- list(name="make_cast_default", TIA=TIA, TOA=TOA)
  retval <- .Call("odp_make_cast_default", TIA, TOA, info, PACKAGE = 'opendpbase')
  returnTran(retval)
}
  
make_count_by_categories <- function(categories, null_category=TRUE, MO="L1Distance", TIA, TOA="int"){
  TIA <- rust_type(categories[1])
  TOA <- toRust(as.character(TOA))
  MO <- sprintf("%s<%s>", MO, TOA)
  
  info <- list(name="make_count_by_categories", categories=categories, null_category=null_category, MO=MO, TIA=TIA, TOA=TOA)
  print(str(info))
  retval <- .Call("odp_make_count_by_categories", categories, null_category, MO, TIA, TOA, info, PACKAGE = 'opendpbase')
  returnTran(retval)
  
}  
# def make_count_by_categories(
#   categories: Any,
#   null_category: bool = True,
#   MO: SensitivityMetric = "L1Distance<int>",
#   TIA: RuntimeTypeDescriptor = None,
#   TOA: RuntimeTypeDescriptor = "int"
# ) -> Transformation:
#   """Make a Transformation that computes the number of times each category appears in the data. 
#     This assumes that the category set is known.
#     
#     [make_count_by_categories in Rust documentation.](https://docs.rs/opendp/latest/opendp/transformations/fn.make_count_by_categories.html)
#     
#     **Citations:**
#     
#     * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#     * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
#     
#     **Supporting Elements:**
#     
#     * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#     * Output Domain:  `VectorDomain<AtomDomain<TOA>>`
#     * Input Metric:   `SymmetricDistance`
#     * Output Metric:  `MO`
#     
#     :param categories: The set of categories to compute counts for.
#     :type categories: Any
#     :param null_category: Include a count of the number of elements that were not in the category set at the end of the vector.
#     :type null_category: bool
#     :param MO: Output Metric.
#     :type MO: SensitivityMetric
#     :param TIA: Atomic Input Type that is categorical/hashable. Input data must be `Vec<TIA>`
#     :type TIA: :py:ref:`RuntimeTypeDescriptor`
#     :param TOA: Atomic Output Type that is numeric.
#     :type TOA: :py:ref:`RuntimeTypeDescriptor`
#     :return: The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
#     :rtype: Transformation
#     :raises TypeError: if an argument's type differs from the expected type
#     :raises UnknownTypeError: if a type argument fails to parse
#     :raises OpenDPException: packaged error from the core OpenDP library
#     """
# assert_features("contrib")
# 
# # Standardize type arguments.
# MO = RuntimeType.parse(type_name=MO)
# TIA = RuntimeType.parse_or_infer(type_name=TIA, public_example=get_first(categories))
# TOA = RuntimeType.parse(type_name=TOA)
# 
# # Convert arguments to c types.
# c_categories = py_to_c(categories, c_type=AnyObjectPtr, type_name=RuntimeType(origin='Vec', args=[TIA]))
# c_null_category = py_to_c(null_category, c_type=ctypes.c_bool, type_name=bool)
# c_MO = py_to_c(MO, c_type=ctypes.c_char_p)
# c_TIA = py_to_c(TIA, c_type=ctypes.c_char_p)
# c_TOA = py_to_c(TOA, c_type=ctypes.c_char_p)
# 
# # Call library function.
# lib_function = lib.opendp_transformations__make_count_by_categories
# lib_function.argtypes = [AnyObjectPtr, ctypes.c_bool, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
# lib_function.restype = FfiResult
# 
# output = c_to_py(unwrap(lib_function(c_categories, c_null_category, c_MO, c_TIA, c_TOA), Transformation))
# 
# return output
# 

make_clamp <- function(bounds, TA="None") {
  TA <- rust_type(bounds[1])
  bounds <- as.list(bounds) 
  info <- list(name="make_clamp", bounds=bounds, TA=TA)
  retval <- .Call("odp_make_clamp", bounds, TA, info, PACKAGE = 'opendpbase')
  returnTran(retval)
}

make_count <- function(TIA = "int", TO = "int"){
  TIA <- toRust(as.character(TIA))
  TO = toRust(as.character(TO))
  info <- list(name="make_count", TIA = TIA, TO = TO)
  retval <- .Call("odp_make_count", TIA, TO, info, PACKAGE = 'opendpbase')
  returnTran(retval)
  
}

make_bounded_sum <- function(bounds, MI="SymmetricDistance", T="None") {
  T <- rust_type(bounds[1])
  bounds <- as.list(bounds) 
  info <- list(name="make_bounded_sum", bounds=bounds, MI=MI, T=T)
  retval <- .Call("odp_make_bounded_sum", bounds, MI, T, info, PACKAGE = 'opendpbase')
  returnTran(retval)
}

make_sized_bounded_mean <- function(size, bounds, MI="SymmetricDistance", T="None"){
  size <- as.integer(size)
  T <- rust_type(bounds[1])
  bounds <- as.list(bounds) 
  info <- list(name="make_sized_bounded_mean", size=size, bounds=bounds, MI=MI, T=T)
  retval <- .Call("odp_make_sized_bounded_mean", size, bounds, MI, T, info, PACKAGE = 'opendpbase')
  returnTran(retval)
}




print.Transformation <- function(x,...){
  cat(sprintf("OpenDP 'Transformation'"))
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
  
  cat(sprintf("\n\n%s\n",capture.output(x$TransformationPtr)))
}


