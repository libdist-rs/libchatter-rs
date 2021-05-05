# Do the setup on the AWS Server

FILE="${1:-/dev/stdin}"
PVT_IP_FILE=${2:-"scripts/aws/pvt_ips.log"}
IPS_FILE=${3:-"scripts/aws/ips_file.log"}
CLI_IPS_FILE=${4:-"scripts/aws/cli_ips.log"}
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

for((i=0;i<4;i++))
do
  ip=${IPS[$i]}
  ssh arch@$ip 'bash -ls --' <<<"cd libchatter-rs; make artemis"
done

N=3
TESTDIR="testdata/b100-n3"
DELAY=50
CLI_TYPE="client-artemis"
SLEEP_TIME=20
M=1000000
W=1000

for((i=0;i<$N;i++))
do
    ip=${IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs node-synchs-rr node-optsync client-optsync node-artemis client-artemis'
    ssh arch@$ip 'bash -ls --' < scripts/aws/debug.sh $i &> node-$i.log &
done

sleep $SLEEP_TIME

echo "Using M: $M"

client=${IPS[$N]}
ssh arch@$client 'bash -ls --' < scripts/aws/throughput-vs-latency/client.sh $TESTDIR $W $CLI_TYPE $M

for((i=0;i<$N;i++))
do
    ip=${IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs node-synchs-rr node-artemis &>/dev/null' &> aws-client.log &
done
wait