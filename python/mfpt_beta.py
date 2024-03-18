#!/usr/bin/python3
import mfpt_exec
import sys
import analytics
import numpy as np

def main():
    # Get and print the current Git hash
    git_hash = mfpt_exec.get_git_hash()

    command = ' '.join(sys.argv)

    parser = mfpt_exec.get_parser()
    # path to file if a < 0
    parser.add_argument('-f', type=str, required=False)
    args = parser.parse_args()

    if args.a >= 1.0 or args.a <= -1.0:
        print("Invalid a")
        sys.exit(0)

    if args.a < 0:
        calc_beta_smaller_0(args)
    else:
        calc_beta_otherwise(args)

def calc_beta_smaller_0(args):
    print("Unimplemented!")

def calc_beta_otherwise(args):
    sz = (args.end - args.start) / (args.samples-1.0)
    x = np.array([args.start + sz * i for i in range(0,args.samples)]) 
    res = analytics.T(x.copy(),args.a)
    for i in range(len(x)):
        print(x[i], res[i])

if __name__ == "__main__":
    main()