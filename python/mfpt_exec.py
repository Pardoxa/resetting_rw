#!/usr/bin/python3
import numpy as np
import MFPT

list=np.array([i * 0.1 for i in range(1,100)])
res=MFPT.Ta(list,0.9)
print(res)