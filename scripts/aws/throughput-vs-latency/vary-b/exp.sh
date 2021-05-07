# Perform the experiments for the different blocksizes
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

W_List=(200 2000 10000 40000 80000 120000)
SW_List=(200 2000 10000 40000 80000 120000)

W2_List=(800 4000 10000 40000 80000 120000)
SW2_List=(800 2000 10000 40000 80000 120000)

W3_List=(1600 4000 10000 40000 80000 120000)
SW3_List=(1600 4000 10000 40000 80000 120000)

W4_List=(4000 10000 40000 80000 120000)
SW4_List=(4000 10000 40000 80000 120000)

W5_List=(10000 40000  80000 120000)
SW5_List=(10000 40000 80000 120000)

N=3

# ======
# SYNCHS 
# ======
# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "synchs"
#     sleep 2
# done >> $1/b100-synchs-run.log

# for w in ${SW2_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "synchs"
#     sleep 2
# done >> $1/b400-synchs-run.log

# for w in ${SW3_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "synchs"
#     sleep 2
# done >> $1/b800-synchs-run.log

# for w in ${SW4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n$N" $w "synchs"
#     sleep 2
# done >> $1/b2000-synchs-run.log

# for w in ${SW5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n$N" $w "synchs"
#     sleep 2
# done >> $1/b4000-synchs-run.log

# ======
# APOLLO 
# ======
for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "apollo"
    sleep 2
done >> $1/b100-apollo-run.log

for w in ${W2_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "apollo"
    sleep 2
done >> $1/b400-apollo-run.log

for w in ${W3_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "apollo"
    sleep 2
done >> $1/b800-apollo-run.log

# for w in ${W4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b1600-n$N" $w "apollo"
#     sleep 2
# done >> $1/b1600-apollo-run.log

# for w in ${W5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b3200-n$N" $w "apollo"
#     sleep 2
# done >> $1/b3200-apollo-run.log

# =======
# OPTSYNC
# =======
for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "optsync"
    sleep 2
done >> $1/b100-optsync-run.log

for w in ${SW2_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "optsync"
    sleep 2
done >> $1/b400-optsync-run.log

for w in ${SW3_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "optsync"
    sleep 2
done >> $1/b800-optsync-run.log

# for w in ${SW4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b1600-n$N" $w "optsync"
#     sleep 2
# done >> $1/b1600-optsync-run.log

# for w in ${SW5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b3200-n$N" $w "optsync"
#     sleep 2
# done >> $1/b3200-optsync-run.log

# =========
# SYNCHS-RR
# =========
# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "synchs-rr"
#     sleep 2
# done >> $1/b100-synchs-rr-run.log

# for w in ${SW2_List[@]}; do
#     echo "DP[Window]: $w"
#     M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "synchs-rr"
#     sleep 2
# done >> $1/b400-synchs-rr-run.log

# for w in ${SW3_List[@]}; do
#     echo "DP[Window]: $w"
#     M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "synchs-rr"
#     sleep 2
# done >> $1/b800-synchs-rr-run.log

# for w in ${SW4_List[@]}; do
#     echo "DP[Window]: $w"
#     M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n$N" $w "synchs-rr"
#     sleep 2
# done >> $1/b2000-synchs-rr-run.log

# for w in ${SW5_List[@]}; do
#     echo "DP[Window]: $w"
#     M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n$N" $w "synchs-rr"
#     sleep 2
# done >> $1/b4000-synchs-rr-run.log

# =============
# APOLLO NORMAL
# =============
# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/b100-apollo-normal-run.log

# for w in ${W2_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/b400-apollo-normal-run.log

# for w in ${W3_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/b800-apollo-normal-run.log

# for w in ${W4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/b2000-apollo-normal-run.log

# for w in ${W5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n$N" $w "apollo" "normal"
#     sleep 2
# done >> $1/b4000-apollo-normal-run.log

# =======
# ARTEMIS
# =======
for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n$N" $w "artemis"
    sleep 2
done >> $1/b100-artemis-run.log

for w in ${W2_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n$N" $w "artemis"
    sleep 2
done >> $1/b400-artemis-run.log

for w in ${W3_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n$N" $w "artemis"
    sleep 2
done >> $1/b800-artemis-run.log

# for w in ${W4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b1600-n$N" $w "artemis"
#     sleep 2
# done >> $1/b1600-artemis-run.log

# for w in ${W5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b3200-n$N" $w "artemis"
#     sleep 2
# done >> $1/b3200-artemis-run.log
