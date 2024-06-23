# Exhaustively checks each 32-bit float between [0, inf) to find the greatest error (in ulps) of the erfc function.
# The result from this check is that the erfc function in statrs errs by at most 1 f32 ulp.

# First disagreement is at 0.5.
# Execution is slowest at inputs around 15, before switching to the next approximating curve.

# To use all CPUs, floats are sharded modulo the number of CPUs (less two)

import struct
import multiprocessing

# pip install gmpy2
import gmpy2

# add the following to rust/src/data/ffi/mod.rs and recompile
# #[bootstrap(name = "erfc")]
# /// Internal function. Compute erfc.
# #[no_mangle]
# pub extern "C" fn opendp_data__erfc(value: f64) -> f64 {
#     use statrs::function::erf::erfc;
#     erfc(value)
# }
from opendp._data import erfc


# specifically check max ulp distance from a conservative upper bound
gmpy2.get_context().round = gmpy2.RoundUp


def floatToBits(f):
    s = struct.pack(">f", f)
    return struct.unpack(">l", s)[0]


def bitsToFloat(b):
    s = struct.pack(">l", b)
    return struct.unpack(">f", s)[0]


def worker(offset, step):
    max_err = 0
    print(f"running {offset}")
    # iterate through all 32-bit floats
    for bits in range(floatToBits(0.0), floatToBits(float("inf")), step):
        bits += offset
        if offset == 0 and bits > 0 and (bits // step) % 10_000 == 0:
            prop_done = bits / floatToBits(float("inf"))
            print(
                f"{prop_done:.2%} done, with max discovered f32 ulp error of: {max_err}. Currently at: {bitsToFloat(bits)}"
            )

        f32 = bitsToFloat(bits)
        f32_ulp_err = abs(floatToBits(erfc(f32)) - floatToBits(gmpy2.erfc(f32)))
        max_err = max(max_err, f32_ulp_err)
    print(max_err)


if __name__ == "__main__":
    n_cpus = multiprocessing.cpu_count() - 2
    processes = []
    for cpu in range(n_cpus):
        p = multiprocessing.Process(target=worker, args=(cpu, n_cpus))
        p.start()
        processes.append(p)

    [p.join() for p in processes]


# max_err = 0
# # edges taken from statrs source
# edges = [0., 1e-10, 0.5, 0.75, 1.25, 2.25, 3.5, 5.25, 8.0, 11.5, 17.0, 24.0, 38.0, 60.0, 85.0, 110.0]
# from math import nextafter
# def check_err(v):
#     print(v, abs(floatToBits(erfc(v)) - floatToBits(gmpy2.erfc(v))))
# for edge in edges:
#     check_err(nextafter(edge, -1))
#     check_err(edge)
