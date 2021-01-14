# An experiments needs the following things set up first
#
# cli_ip_file: A file for the client consisting of all the pvt ips on where to
# contact the nodes of the protocol
#
# ip_file: A file containing the pvt ips of other nodes. Used by the server to
# contact the other servers

IN_FILE=${1:-"scripts/aws/aws_ips.log"}
TESTDIR=${2:-"testdata/b100-n3"}
W=${3:-"50000"}
TYPE=${4:-"apollo"}
N=${5:-"3"}
DELAY=${6:-"50"}

# echo "Using: $TYPE $TESTDIR $DELAY"

while IFS= read -r line; do
    ACTUAL_IPS+=($line)
done < $IN_FILE

for((i=0;i<$N;i++))
do
    ip=${ACTUAL_IPS[$i]}
    echo "Setting up: $ip"
    ssh arch@$ip 'killall node-apollo node-synchs'
    # sleep 1
    ssh arch@$ip 'bash -ls --' < scripts/aws/throughput-vs-latency/$TYPE.sh $i $TESTDIR $DELAY &
done

sleep 60

client=${ACTUAL_IPS[$N]}
ssh arch@$client 'bash -ls --' < scripts/aws/throughput-vs-latency/client.sh $TESTDIR $W $TYPE

for((i=0;i<$N;i++))
do
    ip=${ACTUAL_IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs'
done