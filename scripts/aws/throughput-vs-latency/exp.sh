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
# TYPE is one of ["apollo", "synchs", "synchs-rr", "optsync", "artemis"]
TYPE=${4:-"apollo"}
# CLI_TYPE is one of ["default","client-apollo","client-synchs","normal-client-apollo", "artemis"]
CLI_TYPE=${5:-"default"}
DELAY=${6:-"50"}
M=${M:-"1000000"}
SLEEP_TIME=10

if [ $TYPE == "synchs" ]; then 
    CLI_TYPE="client-$TYPE"
elif [ $TYPE == "synchs-rr" ] ; then
    M="100000"
    CLI_TYPE="client-synchs"
elif [ $TYPE == "apollo" ]; then
    if [ $CLI_TYPE == "default" ]; then 
        CLI_TYPE="client-apollo"
    else 
        CLI_TYPE="normal-client-apollo"
    fi
elif [ $TYPE == "optsync" ]; then
    CLI_TYPE="client-$TYPE"
fi

while IFS= read -r line; do
    ACTUAL_IPS+=($line)
done < $IN_FILE

N=3

for((i=0;i<$N;i++))
do
    ip=${ACTUAL_IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs node-synchs-rr node-optsync client-optsync'
    ssh arch@$ip 'bash -ls --' < scripts/aws/throughput-vs-latency/$TYPE.sh $i $TESTDIR $DELAY $CLI_TYPE &
done

sleep $SLEEP_TIME

echo "Using M: $M"

client=${ACTUAL_IPS[$N]}
ssh arch@$client 'bash -ls --' < scripts/aws/throughput-vs-latency/client.sh $TESTDIR $W $CLI_TYPE $M

for((i=0;i<$N;i++))
do
    ip=${ACTUAL_IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs node-synchs-rr'
done