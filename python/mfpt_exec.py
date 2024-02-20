#!/usr/bin/python3
import numpy as np
import MFPT
import argparse
import sys
import subprocess
import os

def get_git_hash():
    try:
        # Get the real path of the script file being executed
        this_files_path = os.path.realpath(__file__)
        # Extract the directory path of the script
        git_directory=os.path.dirname(this_files_path)
        print(git_directory)
        # Execute git command to get the commit hash
        result = subprocess.run(['git', 'rev-parse', 'HEAD'], stdout=subprocess.PIPE, cwd=git_directory)
        # Decode the output and strip any trailing whitespace
        git_hash = result.stdout.decode('utf-8').strip()
        return git_hash
    except Exception as e:
        print("Error:", e)
        return None

# Get and print the current Git hash
git_hash = get_git_hash()

command = ' '.join(sys.argv)

parser = argparse.ArgumentParser(
    prog="analytical mean first passage time",
    description="prints out the analytical mean first passage time"
)
parser.add_argument('-s', '--start', type=float, required=True)
parser.add_argument('-e', '--end', type=float, required=True)
parser.add_argument('--samples', required=True, type=int)
parser.add_argument('-a', type=float, required=True)
args = parser.parse_args()

sz = (args.end - args.start) / (args.samples-1.0)
x = np.array([args.start + sz * i for i in range(0,args.samples)]) 
mfpt_arr=MFPT.Ta(x, args.a)
print("#", command)
if git_hash:
    print("# Current Git hash:", git_hash)
else:
    print("# Failed to retrieve Git hash.")

for rate, mfpt in zip(x, mfpt_arr):
    print(rate, " ", mfpt)