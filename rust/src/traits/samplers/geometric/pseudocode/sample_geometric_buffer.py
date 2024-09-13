# type: ignore
def sample_geometric_buffer(
    buffer_len: usize, constant_time: bool
) -> Optional[uint]:  # |\label{line:geombuffer}|
    if constant_time:
        buf = bytearray(buffer_len)
        fill_bytes(buf)  # mutates in-place
        ret = None
        for i in range(buffer_len):
            # find first nonzero event
            if buf[i] > 0:
                # compute index of first nonzero bit buffer
                cand = 8 * i + buf[i].leading_zeroes()  # |\label{line:indexcmp}|
                ret = cand if ret is None else min(ret, cand)
        return ret
    else:
        for i in range(buffer_len):
            buf = bytearray(1)
            fill_bytes(buf)  # mutates in-place
            if buf[0] > 0:
                return 8 * i + buf[0].leading_zeroes()

        return None
