from opendp.accuracy import *


def print_statement(dist, scale, accuracy, alpha):
    perc = round((1 - alpha) * 100)
    print(f"When the {dist} scale is {scale}, the DP estimate differs from the true value "
          f"by no more than {accuracy} at a level-alpha of {alpha}, "
          f"or with (1 - {alpha})100% = {perc}% confidence.")


def test_laplacian_scale_to_accuracy():
    def check_laplacian_scale_to_accuracy(scale, alpha):
        print_statement("laplacian", scale, laplacian_scale_to_accuracy(scale, alpha), alpha)

    check_laplacian_scale_to_accuracy(scale=1., alpha=0.05)
    check_laplacian_scale_to_accuracy(scale=2., alpha=0.05)
    check_laplacian_scale_to_accuracy(scale=0., alpha=0.55)


def test_accuracy_to_laplacian_scale():
    def check_accuracy_to_laplacian_scale(accuracy, alpha):
        print_statement("laplacian", accuracy_to_laplacian_scale(accuracy, alpha), accuracy, alpha)

    check_accuracy_to_laplacian_scale(accuracy=1., alpha=0.05)
    check_accuracy_to_laplacian_scale(accuracy=2., alpha=0.05)
    check_accuracy_to_laplacian_scale(accuracy=0.01, alpha=0.1)


def test_gaussian_scale_to_accuracy():
    def check_gaussian_scale_to_accuracy(scale, alpha):
        print_statement("gaussian", scale, gaussian_scale_to_accuracy(scale, alpha), alpha)

    check_gaussian_scale_to_accuracy(scale=1., alpha=0.05)
    check_gaussian_scale_to_accuracy(scale=2., alpha=0.10)
    check_gaussian_scale_to_accuracy(scale=3., alpha=0.55)


def test_accuracy_to_gaussian_scale():
    def check_accuracy_to_gaussian_scale(accuracy, alpha):
        print_statement("gaussian", accuracy_to_gaussian_scale(accuracy, alpha), accuracy, alpha)

    check_accuracy_to_gaussian_scale(accuracy=1., alpha=0.05)
    check_accuracy_to_gaussian_scale(accuracy=2., alpha=0.05)
    check_accuracy_to_gaussian_scale(accuracy=1.2, alpha=0.1)


def test_accuracy_to_discrete_gaussian_scale():
    assert accuracy_to_discrete_gaussian_scale(1.0, 0.5) == 0.797878994872694 # TODO: Closed form expression


def test_accuracy_to_discrete_laplacian_scale():
    assert accuracy_to_discrete_laplacian_scale(1.0, 0.5) == 0.9102392266268373 # TODO: Closed form expression


def test_discrete_gaussian_scale_to_accuracy():
    assert discrete_gaussian_scale_to_accuracy(1.0, 0.5) == 2.0
