from setuptools import setup  # type: ignore
import os

if not os.path.isdir("src/opendp/lib") and os.path.isdir("src/opendp/rust"):
    try:
        from setuptools_rust import RustExtension, Binding  # type: ignore[import-not-found]
    except ImportError:
        raise ImportError(
            "A binary wheel is not available for your platform. Building from source instead. Please run `pip install setuptools-rust`"
        )

    setup(
        rust_extensions=[
            RustExtension(
                # for reference, target=path/in/package/[BINARY_NAME]
                target="opendp/lib/opendp",
                path="src/opendp/rust/Cargo.toml",
                args=["--color", "always"],
                features=["untrusted", "ffi", "polars"],
                binding=Binding.NoBinding,
            )
        ]
    )
else:
    setup()
