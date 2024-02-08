# type: ignore
# returns a single bit with some probability of success
def sample_uniform_int_below(upper, T) -> int:
    while True:
        sample = T.sample_uniform_int()
        if sample < T.MAX - T.MAX % upper:
            return sample % upper
