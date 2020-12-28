#!/usr/bin/python3

# A script that takes raw output from our ping experiment script and produces a
# CSV file that can be used by others for other things.
#
# Usage
# Raw input files as input or stdin
# Output csv files
# What data to extract? 
# For the ping experiment, the options are:
#   - ping times
#   - interval counts
import argparse
import sys
from numpy import percentile
from csv import writer

def filter_ping(s: str):
    if s.count("DP[Time]:") == 1:
        return s.split(":")[1].strip()
    else:
        return None

def filter_interval(s: str):
    if s.count("DP[Int]:") == 1:
        return s.split(":")[1].strip()
    else:
        return None

def interval_process(data):
    return data

def ping_process(data):
    points = [1,10,50,90,95,99,99.1,99.9,99.99,99.999,99.9999,99.99999]
    y = list(percentile(data, points))
    return zip(points, y)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Program to convert raw data of a ping experiment run into a CSV file for plotting")
    parser.add_argument('input', nargs='?', type=argparse.FileType('r'),
                    default=sys.stdin)
    parser.add_argument('output', nargs='?', type=argparse.FileType('w'),
                    default=sys.stdout)
    parser.add_argument('--extract','-e', choices=["ping", "interval"], required=True)
    args = parser.parse_args()
    filter_func = filter_ping
    out_func = ping_process
    if args.extract == "interval":
        # print("Unimplemented")
        filter_func = filter_interval
        out_func = interval_process
        # exit(0)
    counter = 1
    data = []
    for line in args.input:
        if (val := filter_func(line)) != None:
            data.append(int(val))
    processed_data = out_func(data)
    outfile = writer(args.output)
    for d in processed_data:
        outfile.writerow(d)
