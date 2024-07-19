# CCS'24 Artificat Evaluation

Paper: "A Framework for Differential Privacy Against Timing Attacks"

### Installation
1. Install the Rust toolchain: https://www.rust-lang.org/tools/install
2. Make sure you have the latest version by running `rustup update` from a terminal
3. Clone the OpenDP repo:
`git clone git@github.com:zachratliff/opendp.git && cd opendp/rust`
4. Run the Laplace Timing Delay tests
`cargo test --package opendp --lib --features untrusted --features bindings -- --nocapture -- combinators::laplace_delay::test::test_laplace_delay --exact --show-output`


For more detailed instructions on installing Rust and OpenDP you can refer to the official OpenDP user guide: https://docs.opendp.org/en/stable/contributing/development-environment.html

#### Expected output

When you run the OpenDP tests you should expect to see output like the following:
```
running 1 test
release: 5
combined loss: 2.0
combined loss: (2.1, 5.017468205617538e-5)
test combinators::laplace_delay::test::test_laplace_delay ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 216 filtered out; finished in 0.01s
```
