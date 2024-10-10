# type: ignore
def sample_uniform_ubig_below(upper: UBig) -> UBig:
    byte_len = upper.bit_len().div_ceil(8)
    max = Ubig.from_be_bytes([u8.MAX] * byte_len)
    threshold = max - max % upper

    buffer = [0] * byte_len
    while True:
        fill_bytes(buffer)

        sample = UBig.from_be_bytes(buffer)
        if sample < threshold:
            return sample % upper
