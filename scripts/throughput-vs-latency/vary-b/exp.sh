# Run the experiment for w=100,500, 1000, 2000, 4000, 7000,10000, 20000, 40000,
# 60000, 80000

if [ $# -ne 1 ]; then
    echo "Please specify a run prefix"
    echo "Usage: $0 <Run prefix>"
    exit 1
fi

W_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000)
SW_List=(200 500 1000 2000 4000 7000 10000 20000 40000 60000 80000)

W2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)
SW2_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)

W3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000)
SW3_List=(1600 2000 4000 7000 10000 20000 40000 60000 80000)

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    W=$w bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> $1/b100-synchs-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    W=$w bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> $1/b100-apollo-run.log

for w in ${SW2_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3" bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> $1/b400-synchs-run.log

for w in ${W2_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3" bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> $1/b400-apollo-run.log

for w in ${SW3_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b800-n3" bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> $1/b800-synchs-run.log

for w in ${W3_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b800-n3" bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> $1/b800-apollo-run.log

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b100-synchs-run.log $1/b100-synchs.csv

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b100-apollo-run.log $1/b100-apollo.csv

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b400-synchs-run.log $1/b400-synchs.csv

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b400-apollo-run.log $1/b400-apollo.csv

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b800-synchs-run.log $1/b800-synchs.csv

python scripts/throughput-vs-latency/vary-b/parse-exp.py $1/b800-apollo-run.log $1/b800-apollo.csv