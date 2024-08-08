:orphan:
# unit-of-privacy
>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")

>>> privacy_unit = dp.unit_of(contributions=1)
>>> input_metric, d_in = privacy_unit

# /unit-of-privacy


# privacy-loss
>>> privacy_loss = dp.loss_of(epsilon=1.)
>>> privacy_measure, d_out = privacy_loss

# /privacy-loss


# public-info
>>> bounds = (0.0, 100.0)
>>> imputed_value = 50.0

# /public-info


# mediate
>>> from random import randint
>>> data = [float(randint(0, 100)) for _ in range(100)]
>>> context = dp.Context.compositor(
...     data=data,
...     privacy_unit=privacy_unit,
...     privacy_loss=privacy_loss,
...     split_evenly_over=3
... )

# /mediate


# count
>>> count_query = (
...     context.query()
...     .count()
...     .laplace()
... )

>>> scale = count_query.param()
>>> scale
3.0000000000000004

>>> accuracy = dp.discrete_laplacian_scale_to_accuracy(scale=scale, alpha=0.05)
>>> accuracy
9.445721638273584

>>> dp_count = count_query.release()
>>> interval = (dp_count - accuracy, dp_count + accuracy)

# /count


# mean
>>> mean_query = (
...     context.query()
...     .clamp(bounds)
...     .resize(size=dp_count, constant=imputed_value)
...     .mean()
...     .laplace()
... )

>>> dp_mean = mean_query.release()

# /mean
