import math
from itertools import chain, combinations
from typing import NamedTuple

import opendp.prelude as dp
np = dp.import_optional_dependency("numpy")

dp.enable_features("numpy", "contrib", "honest-but-curious")

class AimArgs(NamedTuple):
    '''
    this is just to make the code cleaner I think, I can eliminate this if needed
    '''
    queries: list[list[int]] # W; the workload; each query is a list of integers, which are column indices
    query_weights: list[int] # array of c_s's; each weight applies to a single query
    rho: float               # privacy parameter
    max_size : int           # max memory to feed into Dr. McKenna's algorithm
    alpha: float             # used to calculate sigmas

def make_ordinal_aim(
        input_domain: dp.numpy.array2_domain, 
        input_metric: dp.symmetric_distance,
        args: AimArgs
        ):
    '''
    constructor that implements aim for ordinal data
    '''
    def aim_mechanism(data):
        '''
        this is AIM
        '''
        d = data.shape[1] # number of cols
        n = data.shape[0] # number of rows
        T = 16 * d                                              # Algo 2 line 3
        rho_used = 0                                            # Algo 2 line 5
        t = 0                                                   # Algo 2 line 6
        
        # important arrays updated outside of the loop
        W_plus = [list(subquery) for query in args.queries for subquery in get_all_subsets(query)] # W_plus; the set of all possible queries and subqueries of the workload
        marginal_weights = [] # array of w_r for each r in W_plus
        n_r_array = [] # the number of possible combos of values in the columns specified by each r in W_plus; NEED_TO_INITIALIZE
        sigmas = [math.sqrt(T / (2 * args.alpha * args.rho))]   # Algo 2 line 4
        epsilons = [0] # I initialize this with 0 bc there is no epsilon 0 and want to keep indices consistent with sigmas
        all_marginals = [] # M_r(D) for all r in W_plus; this is a list of lists; NEED_TO_INITIALIZE
        selected_queries = [] # r_1 through r_t-1
        selected_query_indices = [] # indices of queries in selected_queries
        selected_noisy_marginals = [] # this is an array of arrays y_t tilde

        # updated synthetic datasets
        p_hat = np.zeros((n, d)) # the last generated synthetic dataset
        p_hat_new = np.zeros((n, d)) # the new synthetic dataset after each measurement

        # n_r_array initialization
        for query in W_plus:
            n_r = math.prod(input_domain.cardinalities[col_index] for col_index in query)
            n_r_array.append(n_r)
            
        # all_marginals initialization


        # Algo 3: initialize p_t
        size_one_queries, size_one_indices = zip(*[
            (query, index) for index, query in enumerate(W_plus) if len(query) == 1                                 # Algo 3 line 1
        ])
        size_one_measurements = []
        for query, index in zip(size_one_queries, size_one_indices):
            t += 1                                                                                                  # Algo 3 line 2
            sigmas.append(sigmas[0])                                                                                # Algo 3 line 2
            y_t = all_marginals[index] + np.random.normal(0, scale = sigmas[t], size = all_marginals[index].shape)  # Algo 3 line 3
            size_one_measurements.append(y_t)                                                                       # Algo 3 line 3; this array should be reset every time we start measuring again
            rho_used += 1/(2 * sigmas[t]**2)                                                                        # Algo 3 line 4
        p_hat = np.zeros((n, d))                                                                                    # Algo 3 line 6 -- FIX_THIS; need to add the import from Private-PGM

        # calculate marginal weights, Algo 2 line 8
        for query in W_plus:
            w_r = sum(weight * len(set(query), set(s)) for weight, s in zip(args.query_weights, args.queries))
            marginal_weights.append(w_r)
        sigmas.append(sigmas[0])
        epsilons.append(math.sqrt(8 * (1 - args.alpha) * args.rho / args.T)) # Algo 2 line 9

        # large while loop, Algo 2 line 10
        while rho_used < args.rho:
            # important arrays within the loop
            C_t = []
            C_t_indices = [] # indices in W_plus of each query in C_t
            q_r_array = [] # this is an array of length |C_t| where each element has n entries

            t += 1 # Algo 2 line 11; goes to the next index in epsilons and sigmas array that was just added
            rho_used += (epsilons[t]**2 / 8 + 1 / (2 * sigmas[t]**2)) # Algo 2 line 12
            C_t, C_t_indices = zip(*[(query, index) for index, query in enumerate(W_plus)
                                     if query not in selected_queries 
                                     and get_new_query_fit(selected_queries, query, rho_used)]) # Algo 2 line 13
            
            # calculate the q_r(D) for each r in C_t; Algo 2 line 14 gettign each part of the score function
            for query, index in zip(C_t, C_t_indices):
                w_r = marginal_weights[index]
                M_r_D = all_marginals[index]
                M_r_p_hat = get_distribution_marginal(query, p_hat)
                q_r = w_r * (np.sum(np.abs(M_r_D - M_r_p_hat)) - math.sqrt(2 / math.pi) * sigmas[t] * n_r_array[index])
                q_r_array.append(q_r)

            # SELECT the next query using exponential mechanism 
            gumbel_scale = 2 * max(marginal_weights) / epsilons[t] # FIX_THIS
            this_expo_mechanism = dp.m.make_report_noisy_max_gumbel(
                input_domain = dp.vector_domain(dp.atom_domain(float)),
                input_metric = dp.linf_distance(float),
                scale = gumbel_scale,
                optimize = "max"
            )
            selected_query_index = this_expo_mechanism(q_r_array) # use this expo mech to select the index of next query
            selected_query_indices.append(selected_query_index)
            selected_query = C_t[selected_query_index] # Algo 2 line 14; get the actual query from C_t
            selected_queries.append(selected_query) # Algo 2 line 14; add newly selected query to the list

            # MEASURE the marginal on selected query; Algo 2 line 15
            selected_noisy_marginal = all_marginals[selected_query_index] + np.random.normal(0, scale = sigmas[t], size = all_marginals[selected_query_index].shape) 
            selected_noisy_marginals.append(selected_noisy_marginal)

            # ESTIMATE new synthetic dataset using Private-PGM; Algo 2 line 16
            p_hat_new = np.zeros((n, d)) # FIX_THIS; get the actual new thing

            # ANNEAL; Algo 2 line 17; add the new epsilon and sigma to the epsilons and sigmas lists
            new_selected_marginal = get_distribution_marginal(selected_query, p_hat_new)
            old_selected_marginal = get_distribution_marginal(selected_query, p_hat)
            marginals_l1_difference = np.sum(np.abs(new_selected_marginal - old_selected_marginal))
            if marginals_l1_difference <= math.sqrt(2 / math.pi) * sigmas[t] * n_r_array[selected_query_index]:
                epsilons.append(2 * epsilons[t]) # Algo 4 line 2
                sigmas.append(sigmas[t] / 2) # Algo 4 line 3
            else:
                epsilons.append(epsilons[t]) # Algo 4 line 5
                sigmas.append(sigmas[t]) # Algo 4 line 6
            
            rho_diff = args.rho - rho_used
            if rho_diff <= 2 * (1 / (2 * sigmas[t+1]**2) + epsilons[t+1]**2 / 8):
                epsilons[t+1] = 8 * (1 - args.alpha) * rho_diff
                epsilons[t+1] = math.sqrt(epsilons[t+1]) # Algo 4 line 9
                sigmas[t+1] = 1 / (2 * args.alpha * rho_diff)
                sigmas[t+1] = math.sqrt(sigmas[t+1]) # Algo 4 line 10
            
            p_hat = p_hat_new # so that when the loop starts again, we have p_hat_new as the most recently generated one

        synthetic_data = None # FIX_THIS need to get D_hat from p_hat_new using private PGM now
        return synthetic_data
    
    def get_all_subsets(list):
        return chain.from_iterable(combinations(list, r) for r in range(1, len(list) + 1))
    
    def get_new_query_fit(selected_queries, new_query, rho_used):
        '''
        should calculate the junction tree size of this list, then compare it to rho_used/rho * max_size
        '''
        junction_tree_size = 0
        # FIX_THIS: calculate the junction tree size
        new_query_fit = False
        if junction_tree_size <= rho_used / args.rho * args.max_size:
            new_query_fit = True
        return new_query_fit
    
    def get_distribution_marginal(query, distribution):
        '''
        calculates M_r(distribution) in Algo 2 line 14
        '''
        marginal = []
        # FIX_THIS: calculate the marginal
        return marginal
    
    return dp.m.make_user_measurement(
        input_domain = input_domain,
        input_metric = input_metric,
        output_measure = dp.max_divergence(),
        function = aim_mechanism,
        privacy_map = privacy_map, # FIX_THIS; which privacy map to use?
        TO = np.ndarray,
    )