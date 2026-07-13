from __future__ import annotations

import sys
from typing import Optional

from .benchmark import main as benchmark_main


def main(argv: Optional[list[str]] = None) -> None:
    if argv is None:
        argv = sys.argv[1:]
    if "--algorithms" not in argv:
        argv = ["--algorithms", "sklearn,pe_means", *argv]
    benchmark_main(argv)


if __name__ == "__main__":
    main()

