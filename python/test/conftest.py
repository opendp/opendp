# configuration file for pytest that applies to this folder and all subfolders

def pytest_configure(config):
    """hook that is run before executing any tests"""
    import os
    if os.environ.get("OPENDP_RUST_STACK_TRACE_IN_TESTS", "false") != "false":
        import opendp.prelude as dp
        dp.enable_features("rust-stack-trace")
