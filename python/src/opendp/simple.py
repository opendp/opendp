from typing import Any

### SCOPE: Top-level entity, scope for queries (aka Context, Session, Source, Datasource, Dataset, ...)
class Dataset():
    def __init__(self, data, max_contribution=1, max_privacy_loss=None):
        # If max_privacy_loss is None, we get an Odometer
        # Otherwise, we get a Filter
        pass

    def eval(self, query: "Query") -> Any:
        pass

## Scope examples

# Simple scope (Odometer) (we won't use this in the first pass)
scope1 = Dataset([1, 2, 3])

# Scope with max privacy loss (Filter)
scope2 = Dataset([1, 2, 3], max_privacy_loss=1.0)  # Equivalent to max_privacy_loss=Epsilon(1.0)

# budget = Budget(epsilon=10.)  # could say how to compose in here
scope2 = Dataset([1, 2, 3], max_privacy_loss=budget)  # Equivalent to max_privacy_loss=Epsilon(1.0)

# Multiple Scopes sharing the same budget
loss = (1, 0.000001)  # (epsilon, delta)
scope3 = Dataset([1, 2, 3], max_privacy_loss=loss)
scope4 = Dataset([4, 5, 6], max_privacy_loss=loss)


### QUERY: Hook for creating queries using builder-style syntax
class Query():
    def __init__(self):  # DOES QUERY HAVE ANY PARAMS?
        pass

    def __getattr__(self, name):
        # Does something like `self >> make_name()`
        pass


## Query examples

# Inline query
answer1 = scope1.eval(Query().mean().private(epsilon=1.0))
answer1 = scope1.eval(Query().mean().private(loss=PureDP(1.0)))
answer1 = scope1.eval(Query().mean().private(epsilon=1.0, delta=0.00001))
answer1 = scope1.eval(Query().mean().private(loss=ApproxDP(1.0, 0.00001)))

loss = {"epsilon": 1., "delta": 1e-6}
answer1 = scope1.eval(Query().mean().private(**loss))


answer1 = scope1.eval(Query().mean().private(loss=EpsilonDelta(1.0, 0.0001)))

# # Objects for distances - too many!
# PureDP(1.0)      # epsilon
# ApproxDP(...)    # epsilon and delta
#
# zCDP(...)        # rho
# ApproxzCDP(...)  # rho and delta
#
# tCDP(...)        # omega and rho'
# ApproxtCDP()     # omega, rho', delta
#
# RenyiDP(...)     # epsilon'
# ApproxRenyiDP()  # epsilon' and delta
#
# DiscreteDP(...)     # a vector of (ε, δ) tuples






# Discrete query, with accuracy
query2 = Query().mean().private(loss=1.0)
accuracy2 = query2.accuracy(max_contribution=1, alpha=0.1)
answer2 = scope1.eval(query2)



# Discrete query, with specific privacy
query3 = Query().mean().laplace(scale=1.0)
answer3 = scope1.eval(query3)

# Is this valid too?
query4 = Query().mean().laplace().private(loss=1.0)

query4 = Query().mean()
query4 = Query().mean().laplace()


### QUESTIONS
# What name to use for Source/Context/Scope/Dataset/Source/PrivateData/PrivateDataset/Analysis?
# Separate class for measure distances?
# Query or QueryBuilder? Or nothing, just chain constructors?
# Way to capture things like clamping bounds in context? Or is this obviated by "pre built" query elements?
# Where does serialization go?
# Can you attach d_in to Query?

#- Analysis/Budget/Session (max loss for top-level queryable)
#    - Dataset/Context/Session
#        - Query
#        - Query
#    - Dataset/Context/Session
#        - ….



# Query is a partial chain
# Dataset is not a queryable, it is simply a fixture of a dataset arg and a d_in
# Analysis is the root queryable

# USE CASE 1: multi-dataset
analysis = Analysis(budget=PrivacyLoss(epsilon, delta))
dataset1 = Dataset(data1, d_in=DatasetDistance(symmetric=1))
analysis.query(dataset1, query3)

# USE CASE 2: convenience for one dataset
dataset1 = Dataset(data1, d_in=DatasetDistance(symmetric=1), budget=PrivacyLoss(epsilon=1.))
dataset1.eval(query3)


# APPROACH: EVERYTHING IS AN ANALYSIS
top = Analysis(budget=...)
dataset1 = top.eval(Query().dataset(data, d_in=...))  # top.eval() -> top.dataset()
answer1 = dataset1.eval(Query().sum().private(d_out=...))

dataset2 = top.eval(Query().dataset(data, d_in=...))
answer2 = dataset2.eval(Query().mean().private(d_out=...))

single_dataset_analysis = Analysis(budget=..., data=..., d_in=...)

## Andy: why do we need dataset?
# Andy: At some point, something has to wrap the data
# Mike: It also holds d_in
# Mike: Helps to force all queries through one filter/odometer
# Mike: Maps onto common concepts

## Andy: Why are dataset and budget separate?
# Mike: You can have multiple datasets for one budget

## Andy: Doesn't budget hold the data?
# Mike: use make_null_fixture?

# Example of same query re-used
#
mean_query = Query().mean().laplace().private(loss=0.5)

select_age_query = Query().select(column="age")

mean_of_ages_query = select_age_query.mean().laplace().private(loss=0.5)
max_of_ages_query = select_age_query.max().laplace().private(loss=0.5)


dataset1 = Dataset([1, 2, 3], max_privacy_loss=1.0)
dataset2 = Dataset([4, 5, 6], max_privacy_loss=1.0)

ds_answer1 = dataset1.eval(mean_query)
ds_answer2 = dataset2.eval(mean_query)

## START OF Jan 26 SESSION


class PrivacyLoss(object):
    def __init__(self, epsilon=None, delta=None, rho=None):

        self.epsilon = epsilon
        self.delta = delta
        self.rho = rho


class DataDistance(object):
    def __init__(self, contributions=None, changes=None, absolute=None, l1=None):
        pass


# General case: Top-level analysis, multiple children with different datasets
class Analysis(object):
    def __init__(self, budget: PrivacyLoss):
        self.budget = budget

    def eval(self, query) -> Any:
        pass

    def query(self) -> "Analysis":
        pass

    @staticmethod
    def wrap(budget: PrivacyLoss, data, d_in) -> "Analysis":
        return Analysis(budget).eval(Query().dataset(data, d_in=d_in))



data = [1, 2, 3]
budget = PrivacyLoss(epsilon=1.0)
contribution = DataDistance(contributions=1)


top = Analysis(budget=...)
dataset1 = top.eval(Query().wrap(data, d_in=...))  # top.eval() -> top.dataset()
answer1 = dataset1.eval(Query().sum().private(d_out=...))

dataset2 = top.eval(Query().wrap(data, d_in=...))
answer2 = dataset2.eval(Query().mean().private(d_out=...))


dataset2 = top.eval(Query().wrap(data, d_in=...))

single_dataset_analysis = Analysis.wrap(data=data, budget=budget, d_in=...)

# desugars as
single_dataset_analysis = Analysis(budget=PrivacyLoss(epsilon=1)).eval(Query().dataset(data, d_in=...))


# example of using precomputed aggregates
exact_histogram_agg = [1, 2, 5]
exact_histogram_sens = DataDistance(l1=2)
# exact_histogram_qbl = top.eval(Query().dataset(exact_histogram_agg, d_in=exact_histogram_sens))
exact_histogram_qbl = top.query().dataset(exact_histogram_agg, d_in=exact_histogram_sens)
noisy_histogram_agg = exact_histogram_qbl.eval(Query().discrete_laplace(scale=1.))

exact_sum_agg = 23.
exact_sum_sens = DataDistance(absolute=2.)
exact_sum_qbl = top.eval(Query().dataset(exact_sum_agg, d_in=exact_sum_sens))

noisy_sum_agg = exact_sum_qbl.eval(Query().laplace().post(), loss=PrivacyLoss(epsilon=1.0))
# ANDY: ideally we avoid partial chains
query = Query().laplace().post()

noisy_sum_agg = exact_sum_qbl.query().laplace(scale=1.).get()  # TODO: how do we finalize this?
noisy_sum_agg = exact_sum_qbl.query().laplace(loss=PrivacyLoss(epsilon=1.0)).post().get()
noisy_sum_agg = exact_sum_qbl.query().laplace().privatize(loss=PrivacyLoss(epsilon=1.0))
noisy_sum_agg = exact_sum_qbl.query().privatize(loss=PrivacyLoss(epsilon=1.0))
# TODO: rename loss -> cost?

# Without get() by default
# THIS IS IDEAL SO FAR
noisy_sum_agg = exact_sum_qbl.query().laplace(scale=1.)
noisy_sum_agg = exact_sum_qbl.query(epsilon=1.).laplace()
noisy_sum_agg = exact_sum_qbl.query(epsilon=1., eval=False).laplace().post().get()
noisy_sum_agg = exact_sum_qbl.query(epsilon=1., eval=False).laplace(loss).post1().post2().get()
noisy_sum_agg = exact_sum_qbl.query(epsilon=1.).get()

noisy_sum_agg = int_data_qbl.query(epsilon=1.).dp_sum()


# Proposal: spread privacy params and input distances over Analysis args
def proposal_to_spread_privacy_params():
    class Analysis(object):
        def __init__(self, epsilon=None, delta=None, rho=None):
            self.budget = PrivacyLoss(epsilon=epsilon, delta=delta, rho=rho)

        def eval(self, query) -> Any:
            pass

        def query(self) -> "Query":
            pass

        @staticmethod
        def wrap(data, *, contributions=None, changes=None, absolute=None, l1=None, l2=None, **kwargs) -> "Analysis":
            d_in = DataDistance(contributions=contributions, changes=changes, absolute=absolute, l1=l1, l2=l2)
            return Analysis(**kwargs).eval(Query().dataset(data, d_in=d_in))

    # single-dataset analysis
    dataset = Analysis.wrap(data, contributions=2, epsilon=1.)
    dataset.query(epsilon=0.5).sum(bounds=(0, 3))

    # multi-dataset analysis
    root = Analysis(epsilon=1., delta=1e-8)
    dataset1 = root.query().wrap([1, 2, 3, 4], contributions=1)
    _answer1 = dataset1.query(epsilon=0.5).sum(bounds=(0, 3))

    dataset2 = root.query().wrap(23.4, absolute=2.)
    _answer2 = dataset2.query(epsilon=0.1).get()  # .get tells the library to find a suitable mechanism and evaluate it
    _answer3 = dataset2.query(epsilon=0.1).laplace()  # .laplace finalizes the query because it returns a measurement
    _answer4 = dataset2.query().laplace(scale=10.)  # query is fully-determined up-front so no loss is passed into query

    # special case to delay finalization for postprocessing
    _answer5 = dataset2.query(epsilon=0.1, eval=False).laplace().post().get()
    _answer6 = dataset2.query(epsilon=0.1, eval=False).laplace().post1().post2().get()

    # composition with other privacy units
    #   (makes a zCDP odometer IM, wraps with make_zCDP_to_approxDP and make_fix_delta, and passes into root queryable)
    dataset1_zCDP = dataset1.query().zCDP(delta=1e-8)
    _answer7 = dataset1_zCDP.query(rho=0.2).sum()
    _answer8 = dataset1_zCDP.query(rho=0.2).sum()

# TODO: talk about serialization