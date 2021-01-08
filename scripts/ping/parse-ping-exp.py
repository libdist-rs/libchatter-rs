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

def is_message(s: str):
    if s.count("DP[Message]:") == 1:
        return s.split("DP[Message]:")[1].strip()
    return None

def filter_ping(s: str):
    if s.count("DP[Time]:") == 1:
        return s.split("DP[Time]:")[1].strip()
    else:
        return None

def filter_interval(s: str):
    if s.count("DP[Int]:") == 1:
        return s.split("DP[Int]:")[1].strip()
    else:
        return None

def interval_process(data):
    return data

def ping_process(data):
    points = [1,10,50,90,95,99,99.1,99.9,99.99,99.999,99.9999,99.99999]

    out_data = []
    for k in data.keys():
        y = list(percentile(data[k], points))
        zipped = zip(points, y)
        for x,y in zipped:
            out_data.append([k,x,y])
    return out_data

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Program to convert raw data of a ping experiment run into a CSV file for plotting")
    parser.add_argument('input', nargs='?', type=argparse.FileType('r'),
                    default=sys.stdin)
    parser.add_argument('output', nargs='?', type=argparse.FileType('w'),
                    default=sys.stdout)
    args = parser.parse_args()
    filter_func = filter_ping
    out_func = ping_process
    data = {}
    msg = 0
    while (line := args.input.readline()):
        if (d := is_message(line)) != None:
            msg = int(d)
            data[msg] = []
            break
    for line in args.input:
        if (d := is_message(line)) != None:
            msg = d
            data[msg] = []
            continue
        if (val := filter_func(line)) != None:
            data[msg].append(int(val))
    processed_data = out_func(data)
    outfile = writer(args.output)
    for d in processed_data:
        outfile.writerow(d)
