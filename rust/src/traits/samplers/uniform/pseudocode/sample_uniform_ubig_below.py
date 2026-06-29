# type: ignore 
def sample_uniform_ubig_below(upper: UBig) -> UBig: 
    byte_len = upper.bit_len().div_ceil(8) 
    range = 1 << (8 * byte_len) 
    threshold = range - range % upper 

    buffer = [0] * byte_len 
    while True: 
        fill_bytes(buffer) 

        sample = UBig.from_be_bytes(buffer) 
        if sample < threshold: 
            return sample % upper 
