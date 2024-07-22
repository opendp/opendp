'''
The ``extras`` module is a namespace for functionality that requires extra, optional Python dependencies.
'''
from .numpy import np_array2_domain
from .make_np_clamp import make_np_clamp
from .make_np_pca import make_private_np_pca, make_private_np_mean
from .sklearn import PCA