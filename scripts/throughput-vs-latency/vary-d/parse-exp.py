#!/usr/bin/python3

# A script that takes raw output from our ping experiment script and produces a
# CSV file that can be used by others for other things.
#
# Usage
# Raw input files as input or stdin
# Output csv files
import argparse
import sys
from numpy import percentile
from csv import writer

def filter_data(s: str):
    if s.count("DP[Delay]:") == 1:
        return int(s.split("DP[Delay]:")[1].strip())
    elif s.count("DP[Throughput]:") == 1:
        return float(s.split("DP[Throughput]:")[1].strip())
    elif s.count("DP[Latency]:") == 1:
        return float(s.split("DP[Latency]:")[1].strip())
    else:
        return None

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Program to convert raw data of a throught latency experiment run into a CSV file for plotting")
    parser.add_argument('input', nargs='?', type=argparse.FileType('r'),
                    default=sys.stdin)
    parser.add_argument('output', nargs='?', type=argparse.FileType('w'),
                    default=sys.stdout)
    args = parser.parse_args()
    data = []
    while True:
        line1 = args.input.readline()
        if line1 == "":
            break
        line2 = args.input.readline()
        line3 = args.input.readline() 
        window = filter_data(line1)
        throughput = filter_data(line2)
        latency = filter_data(line3) 
        data.append([window, throughput, latency])
    outfile = writer(args.output)
    for d in data:
        outfile.writerow(d)
