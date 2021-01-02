# This scripts converts all raw files in a folder into clean files

for file in $1/* ; do
    fname=`basename $file .log`
    grep "DP\[.*\]:" $file >> "$1"/$fname-cleaned.log
    python scripts/throughput-vs-latency/vary-p/parse-exp.py "$1"/$fname-cleaned.log "$1"/$fname-cleaned.csv
done