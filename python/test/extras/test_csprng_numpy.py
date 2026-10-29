from opendp._lib import get_np_csprng
import ctypes
import multiprocessing
import os
import pytest
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")
scipy = pytest.importorskip("scipy")

try:
    # So randomgen will be in sys.modules, if possible.
    import randomgen  # type: ignore[import-not-found,import-untyped] # noqa F401
except ModuleNotFoundError:
    pass


def test_np_rng():
    n_cats = 100
    n_samples = 100_000

    np_csprng = get_np_csprng()

    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[
        1
    ]
    assert scipy.stats.chisquare(counts).pvalue > 0.0001


@pytest.fixture
def buffered_rng(monkeypatch):
    pytest.importorskip("randomgen")
    import opendp._data as data
    import opendp._lib as lib

    pid = [100]
    fills: list[int] = []

    def fill_bytes(pointer, length):
        values = ctypes.cast(pointer, ctypes.POINTER(ctypes.c_uint64))
        start = (len(fills) + 1) * 10_000
        for index in range(length // 8):
            values[index] = start + index
        fills.append(start)
        return True

    monkeypatch.setattr(lib, "_np_csprng", None)
    monkeypatch.setattr(lib.os, "getpid", lambda: pid[0])
    monkeypatch.setattr(data, "fill_bytes", fill_bytes)
    return get_np_csprng(), pid, fills


def test_np_rng_reuses_buffer_in_same_process(buffered_rng):
    rng, _, fills = buffered_rng
    assert list(rng.bit_generator.random_raw(3)) == [10_000, 10_001, 10_002]
    assert fills == [10_000]


def test_np_rng_discards_inherited_buffer(buffered_rng):
    rng, pid, fills = buffered_rng
    assert rng.bit_generator.random_raw() == 10_000

    pid[0] += 1
    assert rng.bit_generator.random_raw() == 20_000
    assert rng.bit_generator.random_raw() == 20_001
    assert fills == [10_000, 20_000]


def test_np_rng_refills_exhausted_buffer(buffered_rng):
    rng, _, fills = buffered_rng
    assert list(rng.bit_generator.random_raw(1024)) == list(range(10_000, 11_024))
    assert rng.bit_generator.random_raw() == 20_000
    assert fills == [10_000, 20_000]


def test_np_rng_discards_buffer_after_multiple_process_changes(buffered_rng):
    rng, pid, fills = buffered_rng
    assert rng.bit_generator.random_raw() == 10_000
    pid[0] += 1
    assert rng.bit_generator.random_raw() == 20_000
    pid[0] += 1
    assert rng.bit_generator.random_raw() == 30_000
    assert fills == [10_000, 20_000, 30_000]


def _draw_forked_randomness(generator, connection):
    try:
        connection.send((os.getpid(), generator.bit_generator.random_raw(32).tobytes()))
    finally:
        connection.close()


@pytest.mark.skipif(
    "fork" not in multiprocessing.get_all_start_methods(), reason="requires fork"
)
def test_np_rng_does_not_reuse_buffer_after_fork(monkeypatch):
    pytest.importorskip("randomgen")
    import opendp._lib as lib

    monkeypatch.setattr(lib, "_np_csprng", None)
    rng = get_np_csprng()
    rng.bit_generator.random_raw()  # Leave unread randomness for both children to inherit.
    context = multiprocessing.get_context("fork")
    processes = []
    readers = []
    try:
        for _ in range(2):
            reader, writer = context.Pipe(duplex=False)
            process = context.Process(
                target=_draw_forked_randomness, args=(rng, writer)
            )
            readers.append(reader)
            process.start()
            processes.append(process)
            writer.close()

        results = []
        for reader, process in zip(readers, processes):
            assert reader.poll(10), "child did not return random bytes"
            results.append(reader.recv())
            process.join(10)
            assert process.exitcode == 0

        assert len({pid for pid, _ in results}) == 2
        parent_bytes = rng.bit_generator.random_raw(32).tobytes()
        assert len({parent_bytes, *(value for _, value in results)}) == 3
    finally:
        for reader in readers:
            reader.close()
        for process in processes:
            if process.is_alive():
                process.terminate()
            process.join(10)
