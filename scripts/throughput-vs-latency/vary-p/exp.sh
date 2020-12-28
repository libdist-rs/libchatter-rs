# Run the experiment for w=100,500, 1000, 2000, 4000, 7000,10000, 20000, 40000,
# 60000, 80000

if [ $# -ne 1 ]; then
    echo "Please specify a run prefix"
    echo "Usage: $0 <Run prefix>"
    exit 1
fi

W_List=(800 1500 2000 4000 7000 10000 20000 40000 60000 80000)
SW_List=(800 1500 2000 4000 7000 10000 20000 40000 60000 80000)

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3" bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p0-synchs-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3" bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p0-apollo-run.log

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3-p128" bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p128-synchs-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3-p128" bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p128-apollo-run.log

for w in ${SW_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3-p1024" bash scripts/synchs-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p1024-synchs-run.log

for w in ${W_List[@]}; do
    echo "DP[Window]: $w"
    W=$w TESTDIR="testdata/b400-n3-p1024" bash scripts/quick-test.sh | grep "DP\["
    sleep 2
done >> scripts/throughput-vs-latency/vary-p/$1-p1024-apollo-run.log

# python scripts/throughput-vs-latency/vary-p/parse-exp.py scripts/throughput-vs-latency/vary-b/$1-synchs-run.log scripts/throughput-vs-latency/vary-b/$1-synchs.csv

# python scripts/throughput-vs-latency/vary-b/parse-exp.py scripts/throughput-vs-latency/vary-b/$1-apollo-run.log scripts/throughput-vs-latency/vary-b/$1-apollo.csv