# type: ignore
def sample_uniform_uint_below(upper: T) -> T:
    threshold = T.MAX - T.MAX % upper

    while True:
        sample = sample_from_uniform_bytes()
        if sample < threshold:
            return sample % upper
