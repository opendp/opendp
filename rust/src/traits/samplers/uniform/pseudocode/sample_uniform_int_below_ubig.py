# type: ignore
# returns a single bit with some probability of success
def sample_uniform_int_below(upper: int, trials: Optional[int]) -> int:
    byte_len = div_ceil(upper.bit_len(), 8)
    max = Ubig.from_be_bytes([u8.MAX] * byte_len)
    threshold = max - max % upper

    found = None
    buffer = [0] * byte_len
    while True:
        if trials == 0:
            if found is None:
                raise ValueError("failed to sample")
            return found
        trials = None if trials is None else trials - 1

        fill_bytes(buffer)

        sample = UBig.from_be_bytes(buffer)
        if sample < threshold and found is None:
            found = sample % upper
        
        if found is not None and trials is None:
            return found
