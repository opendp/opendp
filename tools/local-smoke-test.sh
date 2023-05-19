#!/bin/bash

# From the repo root, run:
# bash tools/local-smoke-test.sh
#
# This tests the code in the current branch, and exits with nonzero exit code if any tests fail.

# show line numbers
PS4=':$LINENO+'
# exit immediately if any command fails
set -e

LOG_DIR=tools/out/logs

# activate virtualenv
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

last_modified_timestamp() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        find . -type f -printf '%C@' | sort -rn | tail -1
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        find . -print0 | xargs -0 stat -f %m | sort -rn | head -1
    fi
}

# terminate early if no files changed since last run
prev_modified=`cat $LOG_DIR/last_modified.ans 2>/dev/null || echo ""`
next_modified=`last_modified_timestamp`
if [ "$prev_modified" == "$next_modified" ]; then
    echo "no files changed since last run"
    train
    exit 0
fi

# fail immediately if any command fails in a pipe
set -o pipefail

# clear logs
rm -r $LOG_DIR || true
mkdir -p $LOG_DIR

# start timer
start=`date +%s`

# build rust
cargo build --color=always --manifest-path=./rust/Cargo.toml --features untrusted,bindings-python 2>&1 | tee $LOG_DIR/cargo_build.ans

mid=`date +%s`
echo "  🦀 cargo build         ($((mid-start)) s) $LOG_DIR/cargo_build.ans"

# all tasks below this line run in parallel and are compared against the `mid` timestamp

# check that rust code is formatted (use -- --check to get nonzero exit code)
if ! cargo fmt --manifest-path=./rust/Cargo.toml -- --check > $LOG_DIR/cargo_fmt.ans 2>&1
then
    echo "cargo fmt --check failed"
    cat $LOG_DIR/cargo_fmt.ans
    false
fi && echo "  🧹 cargo fmt --check   ($((`date +%s`-mid)) s) $LOG_DIR/cargo_fmt.ans" &
pid_cargo_fmt=$!

# check that windows build works
if ! cargo check --color=always --manifest-path=./rust/Cargo.toml --no-default-features --features untrusted,ffi > $LOG_DIR/cargo_check_windows.ans 2>&1
then
    echo "cargo check windows failed"
    cat $LOG_DIR/cargo_check_windows.ans
    false
fi && echo "  🪟  cargo check windows ($((`date +%s`-mid)) s) $LOG_DIR/cargo_check_windows.ans" &
pid_cargo_check_windows=$!

# python tests
if ! pytest --color=yes python/test > $LOG_DIR/pytest.ans
then    
    echo "pytest failed"
    cat $LOG_DIR/pytest.ans
    false
fi && echo "  🐍 pytest              ($((`date +%s`-mid)) s) $LOG_DIR/pytest.ans" &
pid_pytest=$!

# build html (sleep to avoid https://github.com/sphinx-doc/sphinx/issues/10111)
if ! (sleep 1 && make -C docs html) > $LOG_DIR/make_html.ans 2>&1 
then
    echo "make html failed ($((`date +%s`-mid)) s)"
    cat $LOG_DIR/make_html.ans
    false
fi && echo "  🌎 make html           ($((`date +%s`-mid)) s) $LOG_DIR/make_html.ans" &
pid_make_html=$!

# python doctests
if ! make -C docs doctest-python > $LOG_DIR/make_doctest-python.ans 2>&1 
then
    echo "make doctest-python failed ($((`date +%s`-mid)) s)"
    cat $LOG_DIR/make_doctest-python.ans
    false
fi && echo "  📖 make doctest-python ($((`date +%s`-mid)) s) $LOG_DIR/make_doctest-python.ans" &
pid_make_doctest_python=$!

# type-check python notebooks
if (nbqa pyright docs/source || true) 2>&1 | grep -E "Argument missing for parameter| is not defined" --color=always > $LOG_DIR/nbqa_pyright.ans 2>&1
then
    echo "nbqa pyright failed ($((`date +%s`-mid)) s)"
    cat $LOG_DIR/nbqa_pyright.ans
    false
fi && echo "  📒 nbqa pyright        ($((`date +%s`-mid)) s) $LOG_DIR/nbqa_pyright.ans" &
pid_nbqa_pyright=$!

# run rust tests (color=always is first set for crate compilation and second for test results)
if ! cargo test --manifest-path=./rust/Cargo.toml --features untrusted,bindings-python --color=always -- --color=always > $LOG_DIR/cargo_test.ans 2>&1
then 
    echo "cargo test failed"
    cat $LOG_DIR/cargo_test.ans
    false
fi && echo "  🦞 cargo test          ($((`date +%s`-mid)) s) $LOG_DIR/cargo_test.ans" & 
pid_cargo_test=$!

# run notebooks (in bash to get globstar working)
# if ! bash -c "shopt -s globstar && pytest --color=yes -x --nbmake docs/source/**/*.ipynb -n=auto --durations=5" > $LOG_DIR/pytest_nbmake.ans
# then
#     echo "pytest nbmake failed"
#     cat $LOG_DIR/pytest_nbmake.ans
#     false
# fi && echo "  📓 pytest nbmake       ($((`date +%s`-mid)) s) $LOG_DIR/pytest_nbmake.ans" &
# pid_pytest_nbmake=$!

# separate lines so that exit codes are not ignored
wait $pid_cargo_fmt 
wait $pid_pytest 
wait $pid_cargo_check_windows 
wait $pid_make_html
wait $pid_make_doctest_python 
wait $pid_nbqa_pyright 
wait $pid_cargo_test 
# wait $pid_pytest_nbmake

echo `last_modified_timestamp` > $LOG_DIR/last_modified.ans
echo "done                     ($((`date +%s`-start)) s) $LOG_DIR/last_modified.ans"

train
