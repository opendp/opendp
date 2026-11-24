# type: ignore
def gamma_lo_from_x(x: float):
    expm1_hi = (-x).inf_exp_m1()  # |\label{line:expm1_hi}|
    return -expm1_hi
