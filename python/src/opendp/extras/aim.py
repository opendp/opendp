import numpy as np
import opendp.prelude as dp
import opendp.measurements as dpm
import itertools
import math

# have to get ryan mckenna's library as a dependence
from mbi import estimation, LinearMeasurement 
    # TODO: get this as dependency https://github.com/ryan112358/private-pgm/blob/master/src/mbi/estimation.py

dp.enable_features("contrib")

# can ignore most comments, they're just notes for me to remember reasoning
# how do we determine the domain? is this an input or inferred from the data



class AIM_Mechanism:
    def __init__(self, data, queries, query_weights, rho, MAX_SIZE = 80, alpha = 0.9):
        self.D = data
        self.queries = queries                                              # these are the queries r in the paper that are lists of attribute indices
        self.query_weights = query_weights                                  # these are the weights c_i in the paper
        self.queries_extended = self.get_extended_queries()                 # this is W_plus in the paper
        self.rho = rho
        self.max_size = MAX_SIZE
        self.alpha = alpha

        self.d = data.shape[1]                                              # this assumes data is numpy array -- should i use something else?
        self.T = 16 * self.d
        self.t = 0                                                          # used for counting in algo 3
        self.rho_used = 0                                                   # need to use opendp accountant for this i think
        self.sigma = [math.sqrt(self.T/(2 * self.alpha * self.rho))]        # array of sigmas; sigma[0] is initialized as specified in algo
        
        self.new_t = 0                                                      # algo 2, used for new_sigma and epsilon
        self.new_sigma = []                                                 # to be used in algo 2; sigma_t+1 onwards from line 9
        self.epsilon = []                                                   # from line 9 onwards; changed index so epsilon[t+i] in paper is just epsilon[i] here
        self.already_selected_queries = None                                # used to calculate size remaining
        self.already_selected_marginals = None
        
        self.distribution_estimate = None

    def get_query_subsets(self, s):
        '''outputs list of all subsets of a set s (array/list)'''
        subsets_of_s = [list(subset) for i in range(len(s) + 1) for subset in itertools.combinations(s, i)]
            # itertools.combinations(s, i) gives all sets of size i in s
            # iterate from i to len(s) + 1 to get all possible subset sizes
        return subsets_of_s
    
    def get_extended_queries(self):
        '''get the W_plus'''
        query_subsets_list = [self.get_query_subsets(s) for s in self.queries]
            # each element of this list is a list of subsets for each original query s in the workload W
        queries_extended = [query for subset in query_subsets_list for query in subset]
            # creates a flattened list of all the queries in query_subsets_list
        return queries_extended
    
    def get_real_marginal(self, r):
        '''
        r is a list of the indices for the attribute columns
        OUTPUT: a list of counts for each tuple combo of the attributes for the ACTUAL dataset self.D
        '''
        #TODO: implement this
        #TODO: for this function, I'm confused about how the domain is specified
        relevant_columns = self.D[:, r]
        return [1,2]
    
    def get_approximate_marginal(self, r):
        '''approximate marginals M_r(p hat)'''
        # TODO: implement this; domain confusion
        return 0

    def measure_marginal(self, r, scale):
        '''returns noisy marginal with gaussian noise of std scale'''
        this_gaussian_mechanism = dp.make_gaussian(
            input_domain = dp.atom_domain(float),
            input_metric = dp.l2_distance(float),
            scale = scale
        )
        original_marginal = self.get_real_marginal(r)
        noisy_marginal = [this_gaussian_mechanism(element) for element in original_marginal]
        return noisy_marginal
    
    def estimate_distribution(self):
        measurements_for_optimization = [
            LinearMeasurement(
                self.already_selected_marginals[i],
                self.already_selected_queries[i],
                stddev = self.sigma[i]
            )
            for i in range(len(self.already_selected_queries))
        ]
        distribution_estimate = estimation.mirror_descent(
            domain = self.domain,                                           # TODO: I'm confused about how to define the domain for his function
            loss_fn = measurements_for_optimization,                        # list of linear measurements
            known_total = float(self.D.shape[0])                            # i think this is the number of total rows?
        )
        return distribution_estimate
    
    def initialize_distribution_estimate(self):
        '''get the initial distribution estimate for hat(p_t)'''
        self.already_selected_queries = [r for r in self.queries_extended if len(r) == 1]         # each element r is a list of attributes
        self.already_selected_marginals = []                                                      # array of noisy marginal vectors, self.already_selected_marginals[i] is the marginal vector for self.already_selected_queries[i]
        
        for r in self.already_selected_queries:
            if self.t != 0:
                self.sigma.append(self.sigma[0])                                                  # this is to get sigma[t] = sigma[0]
            
            y_t = self.measure_marginal(r = r, scale = self.sigma[self.t])
            self.already_selected_marginals.append(y_t) 
            
            self.rho_used = self.rho_used + 1/(2 * self.sigma[self.t]**2)                         # TODO: need to update this to use opendp accountant

            self.t += 1                                                                           # put this at the end of the loop instead of the beginning to match indexing in python
        
        # first need to convert these into the "linear measurements" needed for their mirror descent fn
        initial_distribution_estimate = self.estimate_distribution()

        return initial_distribution_estimate
    
    def get_quality_weight(self, r):
        '''return w_r in line 8 of algo 2'''
        w_r = 0
        for i, query in enumerate(self.queries):
            overlap = len(set(r) & set(query))
            w_r += self.query_weights[i] * overlap
        return w_r
    
    def JT_SIZE(self, r_t):
        '''calculates size of the junction tree for the queries in the array self.already_selected_queries plus r_t'''
        # TODO: implement this
        return 0

    def get_quality_score(self, r, w_r):
        ''' calculate q_r for query r (line 14, algo 2)'''
        # TODO: implement this
        marginal_difference = la.norm(self.get_real_marginal(r) - self.get_approximate_marginal(r), ord = 1)
        n_r = 0 # TODO: how do you calculate n_r? --> related to domain question
        q_r = w_r * (marginal_difference - math.sqrt(2/math.pi) * self.new_sigma[self.new_t] * n_r)
        return q_r

    def anneal(self, r_t):
        '''update epsilon and new_sigma'''
        left_expr = la.norm(self.get_real_marginal(r_t) - self.get_approximate_marginal(r_t), ord = 1)
        n_rt = 0 # TODO: how to calculate n_rt?
        right_expr = math.sqrt(2 / math.pi) * self.new_sigma[self.new_t] * n_rt

        if left_expr <= right_expr:
            self.epsilon.append(2 * self.epsilon[-1])
            self.new_sigma.append(self.new_sigma[-1] / 2)
        else:
            self.epsilon.append(self.epsilon[-1])
            self.new_sigma.append(self.new_sigma[-1])
        
        budget_remaining = self.rho - self.rho_used
        upper_lim_budget = 1/(2* self.new_sigma[-1]**2) + 1/8 * self.epsilon[-1]**2
        if (budget_remaining) <= 2 * upper_lim_budget:
            self.epsilon[-1] = math.sqrt(8 * (1 - self.alpha) * budget_remaining)
            self.new_sigma[-1] = math.sqrt(1/(2 * self.alpha * budget_remaining))


    def select_measure_estimate(self):
        self.distribution_estimate = self.initialize_distribution_estimate()
        self.new_sigma.append(self.sigma[0])                                                    # line 9 of algo 2
        self.epsilon.append(math.sqrt(8*(1 - self.alpha) * self.rho / self.T))                  # eps_t+1; first element of self.epsilon
        threshold = self.rho_used / self.rho * self.max_size

        while self.rho_used < self.rho:                                                         # TODO: change to opendp accountant system
            candidate_queries = []
            self.rho_used += (
                1/8 * self.epsilon[self.new_t]**2 + 1/(2 * self.new_sigma[self.new_t]**2)       # line 12 of algo 2
            ) 
            if self.JT_SIZE([]) > threshold:
                # if adding an empty array [] to our already selected queries is too large, end the while loop
                break
            
            for r_t in self.queries_extended:
                if r_t not in self.already_selected_queries:
                    if (self.JT_SIZE(r_t) <= threshold):
                        candidate_queries.append(r_t)

            # calculate the w_r corresponding to each r in candidate_queries
            w_array = [self.get_quality_weight(r) for r in candidate_queries]

            # calculate the q_r
            q_array = [
                self.get_quality_score(candidate_queries[i], w_array[i]) for i in range(len(w_array))
            ]

            # select r_t in candidate_queries from q_r using expo mechanism
            # is it valid to use the gumbel report noisy max here?
            gumbel_scale = 2 * max(w_array) / self.epsilon[self.new_t]
            this_expo_mechanism = dpm.make_report_noisy_max_gumbel(
                input_domain = dp.vector_domain(dp.atom_domain(float)),
                input_metric = dp.linf_distance(float),
                scale = gumbel_scale,
                optimize = "max"
            )
            r_t_index = this_expo_mechanism(q_array)
            r_t = candidate_queries[r_t_index]

            # measure the marginal
            y_t = self.measure_marginal(r_t, self.new_sigma[self.new_t])
            self.already_selected_marginals.append(y_t)

            # estimate new data distribution
            self.distribution_estimate = self.estimate_distribution()

            # anneal eps[new_t + 1], new_sigma[new_t + 1]
            self.anneal(r_t)

            self.new_t += 1
            
        # TODO: generate synth data using private-PGM
        synthetic_data = 0
        return synthetic_data