# type: ignore
# returns a single bit with some probability of success
def sample_uniform_int_below(upper, T) -> int:
    while True:
        v = T.sample_uniform_int():
        if v < T.MAX - T.MAX % upper:
            return v % upper
