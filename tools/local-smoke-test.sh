#!/bin/bash
PS4=':$LINENO+'
set -e

ENV=opendp
source $WORKON_HOME/$ENV/bin/activate

train() {
    python3 -c 'print("\
    \n\x1b[38;5;236m                  . \x1b[38;5;240m. \x1b[38;5;244m. \x1b[38;5;244mo \x1b[38;5;248mo \x1b[38;5;252mo \x1b[38;5;254mo \x1b[38;5;255mo     \
    \n\x1b[38;5;166m                           ____ \x1b[97m o    \
    \n\x1b[38;5;166m ______   ______   ______  \x1b[38;5;202m|  |__||_  \
    \n\x1b[38;5;202m|      | |      | |      | | \x1b[38;5;145m    \x1b[38;5;202m   ) \
    \n\x1b[38;5;145m ()\x1b[38;5;221m--\x1b[38;5;145m() \x1b[38;5;253m~ \x1b[38;5;145m()\x1b[38;5;221m--\x1b[38;5;145m() \x1b[38;5;253m~ \x1b[38;5;145m()\x1b[38;5;221m--\x1b[38;5;145m() \x1b[38;5;253m~ \x1b[38;5;145m()\x1b[38;5;221m---\x1b[38;5;145m()\x1b[38;5;221m-/ \033[0m")'
}

prev_modified=`cat out/logs/last_modified.ans 2>/dev/null || echo ""`
next_modified=`zsh -c 'ls -log **/*(.om[1])'`

if [ "$prev_modified" == "$next_modified" ]; then
    echo "no files changed since last run"
    train
    exit 0
fi

set -o pipefail

start=`date +%s`

rm -r out/logs || true
mkdir -p out/logs

# build rust
cargo build --color=always --manifest-path=./rust/Cargo.toml --features untrusted,bindings-python 2>&1 | tee out/logs/cargo_build.ans

mid=`date +%s`
echo "  🦀 cargo build         ($((mid-start)) s) out/logs/cargo_build.ans"

(trap 'kill 0' SIGINT;

# check that rust code is formatted (use -- --check to get nonzero exit code)
if ! cargo fmt --manifest-path=./rust/Cargo.toml -- --check > out/logs/cargo_fmt.ans 2>&1
then
    echo "cargo fmt -- --check failed"
    cat out/logs/cargo_fmt.ans
    false
fi && echo "  🧹 cargo fmt --check   ($((`date +%s`-mid)) s) out/logs/cargo_fmt.ans" &
pid_cargo_fmt=$!

# python tests
if ! pytest --color=yes python/test > out/logs/pytest.ans
then    
    echo "pytest failed"
    cat out/logs/pytest.ans
    false
fi && echo "  🐍 pytest              ($((`date +%s`-mid)) s) out/logs/pytest.ans" &
pid_pytest=$!

# build html
if ! make -C docs html > out/logs/make_html.ans 2>&1 
then
    echo "make html failed ($((`date +%s`-mid)) s)"
    cat out/logs/make_html.ans
    false
fi && echo "  🌎 make html           ($((`date +%s`-mid)) s) out/logs/make_html.ans" &
pid_make_html=$!

# python doctests
if ! make -C docs doctest-python > out/logs/make_doctest-python.ans 2>&1 
then
    echo "make doctest-python failed ($((`date +%s`-mid)) s)"
    cat out/logs/make_doctest-python.ans
    false
fi && echo "  📖 make doctest-python ($((`date +%s`-mid)) s) out/logs/make_doctest-python.ans" &
pid_make_doctest_python=$!

# check that windows build works
if ! cargo check --color=always --manifest-path=./rust/Cargo.toml --no-default-features --features untrusted,ffi > out/logs/cargo_check_windows.ans 2>&1
then
    echo "cargo check windows failed"
    cat out/logs/cargo_check_windows.ans
    false
fi && echo "  🪟  cargo check windows ($((`date +%s`-mid)) s) out/logs/cargo_check_windows.ans" &
pid_cargo_check_windows=$!

# type-check python notebooks
if (nbqa pyright docs/source || true) 2>&1 | grep -E "Argument missing for parameter| is not defined" --color=always > out/logs/nbqa_pyright.ans 2>&1
then
    echo "nbqa pyright failed ($((`date +%s`-mid)) s)"
    cat out/logs/nbqa_pyright.ans
    false
fi && echo "  📒 nbqa pyright        ($((`date +%s`-mid)) s) out/logs/nbqa_pyright.ans" &
pid_nbqa_pyright=$!

# run rust tests
if ! cargo test --manifest-path=./rust/Cargo.toml --features untrusted,bindings-python -- --color=always > out/logs/cargo_test.ans 2>&1
then 
    echo "cargo test failed"
    cat out/logs/cargo_test.ans
    false
fi && echo "  🦞 cargo test          ($((`date +%s`-mid)) s) out/logs/cargo_test.ans" & 
pid_cargo_test=$!

# run notebooks (in bash to get globstar working)
# if ! bash -c "shopt -s globstar && pytest --color=yes -x --nbmake docs/source/**/*.ipynb -n=auto --durations=5" > out/logs/pytest_nbmake.ans
# then
#     echo "pytest nbmake failed"
#     cat out/logs/pytest_nbmake.ans
#     false
# fi && echo "  📓 pytest nbmake       ($((`date +%s`-mid)) s) out/logs/pytest_nbmake.ans" &
# pid_pytest_nbmake=$!

wait $pid_cargo_fmt $pid_pytest $pid_make_doctest_python $pid_make_html $pid_cargo_check_windows $pid_nbqa_pyright $pid_cargo_test $pid_pytest_nbmake
)

echo "done                     ($((`date +%s`-start)) s)"

train

zsh -c 'ls -log **/*(.om[1])' > out/logs/last_modified.ans
