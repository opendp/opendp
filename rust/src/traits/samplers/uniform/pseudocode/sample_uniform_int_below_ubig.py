# type: ignore
# returns a single bit with some probability of success
def sample_uniform_int_below(upper) -> int:
    byte_len = div_ceil(upper.bit_len(), 8)
    max = Ubig.from_be_bytes([u8.MAX] * byte_len)
    threshold = max - max % upper

    buffer = [0] * byte_len
    while True:
        fill_bytes(buffer)

        sample = UBig.from_be_bytes(buffer)
        if sample < threshold:
            return sample % upper
