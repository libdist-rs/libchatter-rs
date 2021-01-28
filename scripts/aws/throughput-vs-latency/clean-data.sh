# This scripts converts all raw files in a folder into clean files

for file in $1/* ; do
    fname=`basename $file .log`
    if [ -e "$1"/$fname-cleaned.log ]; then
        rm -rf "$1"/$fname-cleaned.log
    fi
    grep "DP\[.*\]:" $file >> "$1"/$fname-cleaned.log
    sed -i "/\[Start\]/d" "$1"/$fname-cleaned.log
    sed -i "/\[End\]/d" "$1"/$fname-cleaned.log

    python scripts/throughput-vs-latency/vary-d/parse-exp.py "$1"/$fname-cleaned.log "$1"/$fname-cleaned.csv
done