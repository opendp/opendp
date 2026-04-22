# type: ignore
def sample_poisson(lmbda):
    if lmbda == 0:
        return 0

    pieces = floor(lmbda) + 1  # |\label{line:pieces}|
    piece = lmbda / pieces      # |\label{line:piece}|

    total = 0
    remaining = pieces
    while remaining > 0:
        total += sample_poisson_0_1(piece)  # |\label{line:subcall}|
        remaining -= 1

    return total  # |\label{line:returntotal}|