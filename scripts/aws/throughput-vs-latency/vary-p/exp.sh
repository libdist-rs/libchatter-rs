# Perform the experiments for the different payloads

if [ $# -ne 1 ]; then
    echo "Please specify a run prefix"
    echo "Usage: $0 <Run prefix>"
    exit 1
fi

if [ -e $1 ]; then
    echo "Run directory [$1] already exists"
    # exit 0
fi

mkdir -p "$1"

W_List=(800 7000 20000 40000 80000 120000)
SW_List=(800 7000 20000 40000 80000 120000)

N=3

# ======
# SYNCHS 
# ======

# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "synchs"
#     sleep 2
# done >> $1/p0-synchs-run.log

# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p128" $w "synchs"
#     sleep 2
# done >> $1/p128-synchs-run.log

# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p1024" $w "synchs"
#     sleep 2
# done >> $1/p1024-synchs-run.log

# ======
# APOLLO 
# ======

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "apollo"
    sleep 2
done >> $1/p0-apollo-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p128" $w "apollo"
    sleep 2
done >> $1/p128-apollo-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p1024" $w "apollo"
    sleep 2
done >> $1/p1024-apollo-run.log

# =======
# OPTSYNC
# =======

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "optsync"
    sleep 2
done >> $1/p0-optsync-run.log

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p128" $w "optsync"
    sleep 2
done >> $1/p128-optsync-run.log

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p1024" $w "optsync"
    sleep 2
done >> $1/p1024-optsync-run.log

# =============
# APOLLO NORMAL
# =============
# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/p0-apollo-normal-run.log

# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p128" $w "apollo" "normal"
#     sleep 2
# done >> $1/p128-apollo-normal-run.log

# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p1024" $w "apollo" "normal"
#     sleep 2
# done >> $1/p1024-apollo-normal-run.log

# =======
# Artemis 
# =======

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "artemis"
    sleep 2
done >> $1/p0-artemis-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p128" $w "artemis"
    sleep 2
done >> $1/p128-artemis-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N-p1024" $w "artemis"
    sleep 2
done >> $1/p1024-artemis-run.log
