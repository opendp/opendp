import struct
from opendp._data import erf_inv
from mpmath import mp
import multiprocessing


def floatToBits(f):
    s = struct.pack('>f', f)
    return struct.unpack('>l', s)[0]

def bitsToFloat(b):
    s = struct.pack('>l', b)
    return struct.unpack('>f', s)[0]


def worker(offset, step):
    max_err = 0
    # mp.dps = 20
    print(f"running {offset}")
    for bits in range(floatToBits(1.0), 0, -step):
        bits -= offset
        if offset == 0 and bits > 0 and bits % 1_000 == 0:
            prop_done = bits / floatToBits(1.0)
            print(f"{prop_done:.2%} done, with max discovered ulp error of: {max_err}")

        f64 = bitsToFloat(bits)
        err = abs(floatToBits(erf_inv(f64)) - floatToBits(float(mp.erfinv(f64))))
        max_err = max(max_err, err)
    return max_err

from multiprocessing import Process

if __name__ == "__main__":
    n_cpus = multiprocessing.cpu_count() - 2
    processes = []
    for cpu in range(n_cpus):
        p = Process(target=worker, args=(cpu, n_cpus))
        p.start()
        processes.append(p)

    print([process.join() for process in processes])
    

# from mpmath import libmp
# import numpy as np
# for v in np.random.uniform(size=100):
#     for dps in range(15, 20):
#         mp.dps = dps
#         print(float(mp.erfinv(v)))
# libmp.to_float(x, rnd=libmp.round_ceiling)