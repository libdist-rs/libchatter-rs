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

F=${F:-"1"}

# Test settings for T1 micro
DELAY=(50 100 200 500)
# W=(10000 20000 40000 80000)
W=(40000 80000)
# W=(80000)
# SW=(10000 20000 40000 80000)

N=$(( (2*$F)+1 ))

# for w in ${W[@]}; do
# for d in ${DELAY[@]};do
#     echo "DP[Delay]: $d"
#     bash scripts/aws/throughput-vs-latency/vary-d/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$F" "$w" "optsync" "$N" "$d"
#     sleep 2 # Sleep after an experiment so that the OS releases the socket
# done &>> $1/$w-$F-optsync-run.log
# done

for w in ${W[@]}; do
for d in ${DELAY[@]};do
    echo "DP[Delay]: $d"
    bash scripts/aws/throughput-vs-latency/vary-d/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$F" "$w" "apollo" "$N" "$d"
    sleep 2 # Sleep after an experiment so that the OS releases the socket
done &>> $1/$w-$F-apollo-run.log
done