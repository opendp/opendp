"
# py-init
>>> from opendp.mod import enable_features
>>> enable_features('contrib')

# /py-init
"

# r-init
library(opendp)
enable_features("contrib")
# /r-init

"
# py-demo
>>> import opendp.prelude as dp
>>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
>>> dp_agg = base_laplace(23.4)

# /py-demo
"

# r-demo
space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
base_laplace <- space |> then_base_laplace(1.)
dp_agg <- base_laplace(arg = 23.4)
# /r-demo