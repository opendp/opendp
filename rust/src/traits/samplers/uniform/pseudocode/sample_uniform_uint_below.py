# type: ignore 
def sample_uniform_uint_below(upper: T) -> T: 
    reject_below = (T.MAX % upper + 1) % upper 

    while True: 
        sample = sample_from_uniform_bytes() 
        if sample >= reject_below: 
            return sample % upper 
