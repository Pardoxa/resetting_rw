#!/usr/bin/python3
import mfpt_exec
import sys
import analytics
import numpy as np
import argparse

def main():
    parser = argparse.ArgumentParser(description="MFPT")
    subparsers = parser.add_subparsers(title="subcommands", dest="subcommand")

    # Subparser for float
    g_parser = subparsers.add_parser("greater", help="a>=0")
    g_parser.add_argument('-s', '--start', type=float, required=True)
    g_parser.add_argument('-e', '--end', type=float, required=True)
    g_parser.add_argument('--samples', required=True, type=int)
    g_parser.add_argument('-a', type=float, required=True)
    g_parser.set_defaults(func=calc_beta_otherwise)

    # Subparser for string
    l_parser = subparsers.add_parser("less", help="a<0")
    l_parser.add_argument("-f", type=str, help="file")
    l_parser.add_argument('-a', type=float, required=True)
    l_parser.set_defaults(func=calc_beta_smaller_0)

    args = parser.parse_args()
    if not hasattr(args, "func"):
        parser.print_help()
        exit(1)

    args.func(args)

def calc_beta_smaller_0(args):
    # Using readlines()
    file1 = open(args.f, 'r')
    Lines = file1.readlines()
    for line in Lines:
        if line.startswith("#"):
            continue
        nums = [float(i) for i in line.split()]
        res = analytics.T(nums[0], args.a, boundary=nums[1])
        print(nums[0], res)

def calc_beta_otherwise(args):
    if args.a >= 1.0 or args.a <= -1.0:
        print("Invalid a")
        sys.exit(0)
    
    mfpt_exec.print_git_hash_and_command()
    sz = (args.end - args.start) / (args.samples-1.0)
    x = np.array([args.start + sz * i for i in range(0,args.samples)]) 
    res = analytics.T(x.copy(),args.a)
    for i in range(len(x)):
        print(x[i], res[i])

if __name__ == "__main__":
    main()