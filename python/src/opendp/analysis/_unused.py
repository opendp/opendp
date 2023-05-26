
# class PrivacyLoss(object):

#     def __init__(self, bound=None, measure=None):
#         self.bound = bound
#         self.measure = ty.RuntimeType.parse(measure)
    
#     @staticmethod
#     def epsilon(epsilon, U):
#         return PrivacyLoss(epsilon, ty.MaxDivergence[ty.RuntimeType.parse_or_infer(U, epsilon)])

#     @staticmethod
#     def epsilon_delta(epsilon, delta, U):
#         return PrivacyLoss((epsilon, delta), ty.FixedSmoothedMaxDivergence[ty.RuntimeType.parse_or_infer(U, epsilon)])

#     @staticmethod
#     def rho(rho, U):
#         return PrivacyLoss(rho, ty.ZeroConcentratedDivergence[ty.RuntimeType.parse_or_infer(U, rho)])


# class DataDistance(object):
#     def __init__(self, contributions=None, changes=None, absolute=None, l1=None, l2=None, ordered=False, U=None):
#         kwargs = (x for x in {
#             ("contributions", contributions),
#             ("changes", changes),
#             ("absolute", absolute),
#             ("l1", l1),
#             ("l2", l2)
#         } if x[1] is not None)

#         try:
#             descriptor, self.distance = next(kwargs)
#         except StopIteration:
#             raise ValueError("No distance was specified.")
#         if next(kwargs, None):
#             raise ValueError("At most one distance can be specified.")
        
#         if descriptor == "contributions":
#             self.metric = ty.InsertDeleteDistance if ordered else ty.SymmetricDistance
#         elif descriptor == "changes":
#             self.metric = ty.HammingDistance if ordered else ty.ChangeOneDistance
#         elif descriptor == "absolute":
#             self.metric = ty.AbsoluteDistance[ty.RuntimeType.parse_or_infer(U, self.distance)]
#         elif descriptor == "l1":
#             self.metric = ty.L1Distance[ty.RuntimeType.parse_or_infer(U, self.distance)]
#         elif descriptor == "l2":
#             self.metric = ty.L2Distance[ty.RuntimeType.parse_or_infer(U, self.distance)]
#         else:
#             raise ValueError("Unrecognized distance specification.")
# def make_gaussian():
#     pass

# setattr(make_gaussian, "output_measures", ["ZeroConcentratedDivergence", "RenyiDivergence"])

# print(make_gaussian.output_measures)