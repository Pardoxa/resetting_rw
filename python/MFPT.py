import numpy as np

def tabulate_even_product(a, n):
    '''
    Creates a table containing the values of C_{2k} for k going from 1 to n.
    
    Parameters:
        - (float) a: the scale variable
        - (int)   n: the size of the table to precompute
        
    Returns:
        - List[float] table: An array of size (n+1) where table[j] corresponds to C_{2(j+1)}. 
    '''
    table = np.ones(n+1)
    for k in range(1, n+1):
        table[k] = table[k-1] * (1 - a**(2 * k))
    return table

def tabulate_odd_product(a, n):
    '''
    Creates a table containing the values of C_{2k + 1} for k going from 1 to n.
    
    Parameters:
        - (float) a: the scale variable
        - (int)   n: the size of the table to precompute
        
    Returns:
        - List[float] table: An array of size (n+1) where table[j] corresponds to C_{2j + 1}. 
    '''
    table = np.ones(n+1)
    for k in range(1, n+1):
        table[k] = table[k-1] * (1 - a **(2*k - 1))
    return table

def T(r, even_table, odd_table, cutoff = 100):
    '''
    Computes the theoretical MFPT (as given in Eq. 4 of the latest notes). From precomputed C_{k} tables.
    
    Parameters:
        - (float) or List[float]
            r:             the resetting rate
            
        - List[float]
            even_table:    a precomputed table of C_{2n}
            
        - List[float]
            odd_table:     a precomputed table of C_{2n + 1}
        
    Returns:
        - (float) or List[float]: The MFPT evaluated at all values of "r" which were passed as inputs.
    '''
    out = 0
    
    for n in range(1, cutoff):
        out += 1/(np.math.factorial(2 * n)) * (r ** n) * even_table[n-1]
        
    for n in range(0, cutoff):
        out += 1/(np.math.factorial(2 * n + 1)) * (r ** ((2*n + 1)/2)) * odd_table[n] * even_table[-1] / odd_table[-1]
    
    return 1/r * out

def Ta(r, a, cutoff = 100):
    '''
    A wrapping function to be able to compute directly the MFPT for a given value of "a".
    
    Parameters:
        - (float) or List[float]
            r:             the resetting rate
            
        - (float)
            a:             the scaling variable
        
    Returns:
        - (float) or List[float]: The MFPT evaluated at "a" and all values of "r" which were passed as inputs.
    '''
    even_table = tabulate_even_product(a, cutoff)
    odd_table = tabulate_odd_product(a, cutoff)
    return T(r, even_table, odd_table, cutoff)


def dTdr(r, even_table, odd_table, cutoff = 100):
    '''
    Computes the derivative of the theoretical MFPT with respect to "r". 
    This will be usefull to detect the minimum "r*".
    
    Parameters:
        - (float) or List[float]
            r:             the resetting rate
            
        - List[float]
            even_table:    a precomputed table of C_{2n}
            
        - List[float]
            odd_table:     a precomputed table of C_{2n + 1}
        
    Returns:
        - (float) or List[float]: dT/dr evaluated at all values of "r" which were passed as inputs.
    '''
    out = 0
    
    for n in range(1, cutoff):
        out += 1/(np.math.factorial(2 * n)) * n * (r ** (n-1)) * even_table[n-1]
        
    for n in range(0, cutoff):
        out += 1/(np.math.factorial(2 * n + 1)) * (2*n + 1)/2 * (r ** ((2*n + 1)/2 - 1)) * odd_table[n] * even_table[-1] / odd_table[-1]
    
    return 1/r * out - 1/r * T(r, even_table, odd_table, cutoff)

def rstar(even_table, odd_table, cutoff = 100):
    '''
    Computes the optimal r* from precomputed C_{k} which were created for a specific value of "a".
    
    Parameters:
        - List[float]
            even_table:    a precomputed table of C_{2n}
            
        - List[float]
            odd_table:     a precomputed table of C_{2n + 1}
            
    Returns:
        - (float) rstar:   the optimal r* for which the minimal MFPT is reached.
    '''
    eps = 1e-5
    def rec_find(low, high):
        mid = (low + high)/2
        if high - low < eps:
            return mid
        val = dTdr(mid, even_table, odd_table, cutoff)
        if val < 0:
            return rec_find(mid, high)
        elif val > 0:
            return rec_find(low, mid)
        else:
            return mid
        
    return rec_find(0, 1e3)


def Topt(atab, cutoff = 100):
    
    '''
    Computes the optimal MFPT for specific values of "a".
    
    Parameters:
        - List[float] atab: a list of scaling variables for which to compute the optimal MFPT.
            
    Returns:
        - (List[float], List[float]) 
            (R*, Topt):   A tuple of lists containing respectively the optimal r* at which the minimal MFPT topt is reached.
    '''
    
    Rout = np.zeros(len(atab))
    Tout = np.zeros(len(atab))
    for i, a in enumerate(atab):
        even_table = tabulate_even_product(a, cutoff)
        odd_table = tabulate_odd_product(a, cutoff)
        R = rstar(even_table, odd_table, cutoff)
        Rout[i] = R
        Tout[i] = T(R, even_table, odd_table, cutoff)
    return Rout, Tout

def load_res(a):
    '''
    A short utility to load some of the pre-saved results.
    
    Parameters:
        - (float) a: the value of "a" to reload, must correspond to the filename.
        
    Returns:
        - (List[float], List[float]) (R, T): A tuple of lists of floats. R contains
        the values of the resetting rate "r" for which the MFPT "T" was computed.
        "T" contains the values of the MFPT for the corresponding values of "R".
    '''
    with open('rtab.txt', 'r') as f:
        rtab = np.array(list(map(float, f.readline().split(', '))))
    with open(f'T0({a}).txt', 'r') as f:
        Ttab = np.array(list(map(float, f.readline().split(', '))))
    return rtab, Ttab
