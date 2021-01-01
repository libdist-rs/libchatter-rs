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

W_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000)
SW_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000)

W2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)
SW2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)

W3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000)
SW3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000)

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "synchs"
    sleep 2
done >> $1/b100-synchs-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b100-n3" $w "apollo"
    sleep 2
done >> $1/b100-apollo-run.log

for w in ${SW2_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "synchs"
    sleep 2
done >> $1/b400-synchs-run.log

for w in ${W2_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b400-n3" $w "apollo"
    sleep 2
done >> $1/b400-apollo-run.log

for w in ${SW3_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "synchs"
    sleep 2
done >> $1/b800-synchs-run.log

for w in ${W3_List[@]}; do
    echo "DP[Window]: $w"
    bash scripts/aws/throughput-vs-latency/exp.sh "scripts/aws/aws_ips.log" "testdata/b800-n3" $w "apollo"
    sleep 2
done >> $1/b800-apollo-run.log