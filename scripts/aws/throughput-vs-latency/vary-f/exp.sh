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

F=(1 4 8 16 32)

# Test settings for T1 micro
W=10000
SW=10000
DELAY=50

for f in ${F[@]};do
    N=$(( (2*$f)+1 ))
    echo "DP[Faults]: $f"
    bash scripts/aws/throughput-vs-latency/vary-f/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$f" "$W" "optsync" "$N" "$d"
    sleep 2
done &>> $1/$f-optsync-run.log

for f in ${F[@]};do
    N=$(( (2*$f)+1 ))
    echo "DP[Faults]: $f"
    bash scripts/aws/throughput-vs-latency/vary-f/do_exp.sh "scripts/aws/aws_ips.log" "testdata/b400-p0-f$f" "$W" "apollo" "$N" "$d"
    sleep 2
done &>> $1/$f-apollo-run.log
