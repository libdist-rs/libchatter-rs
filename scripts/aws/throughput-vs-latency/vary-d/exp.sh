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

# W_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)
# SW_List=(800 1000 2000 4000 7000 10000 20000 40000 60000 80000)
F=${F:-"1"}

# Test settings for T1 micro
DELAY=(50 100 200 500)
W=10000
SW=10000

N=$(( (2*$F)+1 ))
for d in ${DELAY[@]};do
    echo "DP[Delay]: $d"
    bash scripts/aws/throughput-vs-latency/vary-d/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$F" "$W" "synchs" "$N" "$d"
    sleep 2
done &>> $1/$F-synchs-run.log

for d in ${DELAY[@]};do
    echo "DP[Delay]: $d"
    bash scripts/aws/throughput-vs-latency/vary-d/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$F" "$W" "apollo" "$N" "$d"
    sleep 2
done >> $1/$F-apollo-run.log