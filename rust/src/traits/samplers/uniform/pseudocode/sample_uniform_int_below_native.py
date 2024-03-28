# type: ignore
# returns a single bit with some probability of success
def sample_uniform_int_below(upper: int, trials: Optional[int]) -> int:
    found = None
    threshold = T.MAX - T.MAX % upper

    while True:
        if trials == 0:
            if found is None:
                raise ValueError("failed to sample")
            return found
        trials = None if trials is None else trials - 1

        sample = T.sample_uniform_int()
        if sample < threshold and found is None:
            found = sample % upper
        
        if found is not None and trials is None:
            return found
