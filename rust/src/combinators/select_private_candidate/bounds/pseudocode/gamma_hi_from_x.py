# type: ignore
def gamma_hi_from_x(x: float):
    expm1_lo = (-x).neg_inf_exp_m1()  # |\label{line:expm1_lo}|
    return -expm1_lo
