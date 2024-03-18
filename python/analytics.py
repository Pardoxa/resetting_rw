import numpy as np

def feven(beta, a, cutoff = 1000):
    '''
    Computes $f_{even}(beta, a)$ by truncating the series up to the cutoff.
    
    Parameters:
        - (float) beta: the scaling variable $beta = \sqrt{\frac{r}{D}} L$. 
        - (float)   a : the jump-scale variable
        
        - Optional[int] cutoff: the index at which to truncate the infinite series.
        
    Returns:
        - (float): The value of $f_{even}(beta, a)$. 
    '''
    beta2 = beta**2
    out = beta2 / 2
    last = beta2 / 2
    for n in range(2, cutoff):
        #print(out, last)
        new = last *  beta2 / ((2*n) * (2*n - 1)) * (1 - a**(2*(n-1)))
        out += new
        last = new
        
    return out


def fodd(beta, a, cutoff = 1000, sign = 1):
    '''
    Computes $f_{odd}(beta, a)$ by truncating the series up to the cutoff.
    
    Parameters:
        - (float) beta: the scaling variable $beta = \sqrt{\frac{r}{D}} L$. 
        - (float)   a : the absolute value |a| of the jump-scale variable
        
        - Optional[int] cutoff: the index at which to truncate the infinite series.
        - Optional[{-1, 1}] sign: the sign of the jump-scale variable.
        
    Returns:
        - (float): The value of $f_{odd}(beta, a)$. 
    '''
    out = beta
    beta2 = beta**2
    last = beta
    for n in range(1, cutoff):
        #print(out, last)
        new = last * beta2 / ((2*n + 1) * (2*n)) * (1 - sign * a**(2*n - 1))
        out += new
        last = new
    return out

def Tpos(beta, a, cutoff = 1000):
    '''
    Computes the dimensionless MFPT $\tilde{T}_a(0) = D T_a(0) / L^2$ with the analytic 
    formula for $0 < a < 1$ by truncating the series at the cutoff.
    
    Parameters:
        - (float) beta: the scaling variable $beta = \sqrt{\frac{r}{D}} L$. 
        - (float)   a : the jump-scale variable
        
        - Optional[int] cutoff: the index at which to truncate the infinite series.
        
    Returns:
        - (float): The value of $T_a(0)$. 
    '''
    def R(a):
        out = 1/(1 - a)
        for i in range(1, cutoff):
            out *= (1 - a**(2*i)) / (1 - a**(2*i + 1))
        return out
    
    return 1/beta**2 * ( feven(beta, a, cutoff) + R(a) * fodd(beta, a, cutoff, sign = 1) )


def Tneg(beta, a, boundary, cutoff = 1000):
    '''
    Computes the dimensionless MFPT $\tilde{T}_a(0) = D T_a(0) / L^2$ with the analytic 
    formula for $-1 < a < 0$ by truncating the series at the cutoff.
    
    Parameters:
        - (float) beta:      the scaling variable $beta = \sqrt{\frac{r}{D}} L$. 
        - (float)   a :      the jump-scale variable
        - (float) boundary:  the value of the MFPT at the boundary $\tilde{T}_a(-L/|a|)$ which has to be found numerically.
                             If no numerical value is available '1' is a relatively ok approximation in some regimes. 
        
        - Optional[int] cutoff: the index at which to truncate the infinite series.
        
    Returns:
        - (float): The value of $T_a(0)$. 
    '''
    
    return (1/beta**2 * 1 / (fodd(beta, a, cutoff, sign = -1) + fodd(beta/a, a, cutoff, sign = -1)) \
            * ( fodd(beta/a, a, cutoff, sign = -1) * feven(beta, a, cutoff) + fodd(beta, a, cutoff, sign = -1) \
               * ( feven(beta/a, a, cutoff) + boundary) ) )

def T(beta, a, boundary = 1, cutoff = 1000):
    '''
    Computes the dimensionless MFPT $\tilde{T}_a(0) = D T_a(0) / L^2$ with the analytic 
    formulas for either $-1 < a < 0$ or $0 < a < 1$ by truncating the series at the cutoff.
    
    Parameters:
        - (float) beta:                the scaling variable $beta = \sqrt{\frac{r}{D}} L$. 
        - (float)   a :                the jump-scale variable
        - Optional[(float)] boundary:  the value of the MFPT at the boundary $\tilde{T}_a(-L/|a|)$ 
                                       which has to be found numerically.
                                       If no numerical value is available '1' is a relatively ok approximation 
                                       in some regimes. 
                                       If a > 0 then this variable is useless and can be left at it's default value.
        
        - Optional[int] cutoff: the index at which to truncate the infinite series.
        
    Returns:
        - (float): The value of $T_a(0)$. 
    '''
    if a > 0:
        return Tpos(beta, a, cutoff)
    elif a < 0:
        return Tneg(beta, -a, boundary, cutoff)
    else:
        raise ValueError('a = 0 not supported.')
        
        