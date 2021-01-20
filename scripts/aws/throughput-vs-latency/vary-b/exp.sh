# Perform the experiments for the different blocksizes
if [ $# -ne 1 ]; then
    echo "Please specify a run prefix"
    echo "Usage: $0 <Run prefix>"
    exit 1
fi

if [ -e $1 ]; then
    echo "Run directory [$1] already exists"
    exit 0
fi

mkdir -p "$1"

W_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)
SW_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)

W2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)
SW2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)

W3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)
SW3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000 100000 120000)

W4_List=(4000 7000 10000 20000 40000 60000 80000 100000 120000)
SW4_List=(4000 7000 10000 20000 40000 60000 80000 100000 120000)

W5_List=(8000 10000 20000 40000 60000 80000 100000 120000 150000)
SW5_List=(8000 10000 20000 40000 60000 80000 100000 120000 150000)


# ================
# Block size: 100
# ================
# for w in ${SW_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "synchs"
#     sleep 2
# done >> $1/b100-synchs-run.log

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "synchs-rr"
    sleep 2
done >> $1/b100-synchs-rr-run.log

# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "apollo"
#     sleep 2
# done >> $1/b100-apollo-run.log

# for w in ${W_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "apollo" "normal"
#     sleep 2
# done >> $1/b100-apollo-normal-run.log

# ================
# Block size: 400
# ================
# for w in ${SW2_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "synchs"
#     sleep 2
# done >> $1/b400-synchs-run.log

for w in ${SW2_List[@]}; do
    echo "DP[Window]: $w"
    M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "synchs-rr"
    sleep 2
done >> $1/b400-synchs-rr-run.log

# for w in ${W2_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "apollo"
#     sleep 2
# done >> $1/b400-apollo-run.log

# for w in ${W2_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "apollo" "normal"
#     sleep 2
# done >> $1/b400-apollo-normal-run.log

# ================
# Block size: 800
# ================
# for w in ${SW3_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "synchs"
#     sleep 2
# done >> $1/b800-synchs-run.log

for w in ${SW3_List[@]}; do
    echo "DP[Window]: $w"
    M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "synchs-rr"
    sleep 2
done >> $1/b800-synchs-rr-run.log

# for w in ${W3_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "apollo"
#     sleep 2
# done >> $1/b800-apollo-run.log

# for w in ${W3_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "apollo" "normal"
#     sleep 2
# done >> $1/b800-apollo-normal-run.log

# ================
# Block size: 2000
# ================
# for w in ${SW4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n3" $w "synchs"
#     sleep 2
# done >> $1/b2000-synchs-run.log

for w in ${SW4_List[@]}; do
    echo "DP[Window]: $w"
    M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n3" $w "synchs-rr"
    sleep 2
done >> $1/b2000-synchs-rr-run.log

# for w in ${W4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n3" $w "apollo"
#     sleep 2
# done >> $1/b2000-apollo-run.log

# for w in ${W4_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b2000-n3" $w "apollo" "normal"
#     sleep 2
# done >> $1/b2000-apollo-normal-run.log

# ================
# Block size: 4000
# ================
for w in ${SW5_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n3" $w "synchs"
    sleep 2
done >> $1/b4000-synchs-run.log

for w in ${SW5_List[@]}; do
    echo "DP[Window]: $w"
    M=100000 bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n3" $w "synchs-rr"
    sleep 2
done >> $1/b4000-synchs-rr-run.log

for w in ${W5_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n3" $w "apollo"
    sleep 2
done >> $1/b4000-apollo-run.log

# for w in ${W5_List[@]}; do
#     echo "DP[Window]: $w"
#     bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b4000-n3" $w "apollo" "normal"
#     sleep 2
# done >> $1/b4000-apollo-normal-run.log
