.onAttach <- function(libname, pkgname) {
    # startup message, we can drop .onAttach in the future
    #Sys.setenv(OPENDP_LIB_DIR="/Users/jago/github/opendp/rust/target/debug/")
    Pver <- read.dcf(file=system.file("DESCRIPTION", package=pkgname),
                      fields="Version")
    packageStartupMessage(paste(pkgname, Pver))
    packageStartupMessage("OpenDP R binding test package")
}

