if __name__ == "__main__":
    import os
    from opendp._lib import proven
    dummy_path = os.path.join(os.path.dirname(__file__), "dummy.tex")
    open(dummy_path, "w").close()

    @proven
    def make_test(a, b):
        """A dummy function for testing the proven decorator.

        [(Proof Document)](dummy.tex))]

        :param a: The first parameter.
        :param b: The second parameter."""
        _ = a, b

    print(make_test.__doc__)
    os.remove(dummy_path)
