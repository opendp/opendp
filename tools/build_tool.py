import argparse
import platform
import subprocess
import sys


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, args, capture_output=True, shell=True):
    if description:
        log(description)
    printed_args = args.join(" ") if type(args) == list else args
    log(printed_args, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(args, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def native_platform():
    plat = platform.system().lower()
    return "macos" if plat == "darwin" else plat


def get_rust_build_command(init, args):
    platform_arg = f" -p {args.platform}" if args.platform is not None else ""
    toolchain_arg = f" -c {args.toolchain}" if args.toolchain is not None else ""
    init_arg = " -i" if init else ""
    release_mode_arg = " -r" if args.release_mode else ""
    tests_arg = " -t" if args.run_tests else ""
    return f"bash tools/rust_build.sh{platform_arg}{toolchain_arg}{init_arg}{release_mode_arg}{tests_arg} -f \"{args.features}\""


def rust_native(args):
    command = get_rust_build_command(False, args)
    run_command("Running Rust build natively", command)


def rust_container(args):
    # This is a stab at doing cross-platform builds in a docker container. It only supports linux,
    # but if we had images for the other platforms, it could work for those in the future.
    platform_to_docker_image = {
        "windows": "windows-latest",
        "macos": "macos-10.15",
        "linux": "quay.io/pypa/manylinux2010_x86_64",
    }
    docker_image = platform_to_docker_image[args.platform]
    mount_point = "/io"
    command = get_rust_build_command(True, args)
    docker_command = f"cd {mount_point} && {command}"
    run_command(f"Building Rust library in a container", f"docker run --rm -v `pwd`:{mount_point} {docker_image} bash -c '{docker_command}'")


def rust(args):
    log(f"*** BUILDING RUST LIBRARY ***")
    if args.platform is None or args.platform == native_platform():
        rust_native(args)
    else:
        rust_container(args)


def python(_args):
    log(f"*** BUILDING PYTHON LIBRARY ***")
    command = "bash tools/python_build.sh"
    run_command("Running Python build", command)


def meta(_args):
    meta_args = [
        "rust",
        "python",
    ]
    for args in meta_args:
        _main(f"meta {args}".split())


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP build tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("rust", help="Build Rust library")
    subparser.set_defaults(func=rust)
    subparser.add_argument("-p", "--platform", choices=["mac", "windows", "linux"])
    subparser.add_argument("-c", "--toolchain", default="stable")
    subparser.add_argument("-r", "--release-mode", dest="release_mode", action="store_true", default=True)
    subparser.add_argument("-nr", "--no-release-mode", dest="release_mode", action="store_false")
    subparser.add_argument("-t", "--run-tests", dest="run_tests", action="store_true", default=True)
    subparser.add_argument("-nt", "--no-run-tests", dest="run_tests", action="store_false")
    subparser.add_argument("-f", "--features", default="untrusted")

    subparser = subparsers.add_parser("python", help="Build Python library")
    subparser.set_defaults(func=python)
    subparser.add_argument("-p", "--platform", choices=["mac", "windows", "linux"])

    subparser = subparsers.add_parser("all", help="Build everything")
    subparser.set_defaults(func=meta, command="all")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
