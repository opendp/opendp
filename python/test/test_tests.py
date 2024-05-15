def test_number_of_tests_found(request):
    tests_found = len(request.session.items)
    assert tests_found >= 304
