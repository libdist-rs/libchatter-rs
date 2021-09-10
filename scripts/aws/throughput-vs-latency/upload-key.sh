# Upload a key to all the servers

# First argument is the IPS file
# Second argument is the key


while IFS= read -r line; do
    ACTUAL_IPS+=($line)
done < $1

N=10

for((i=0;i<$N;i++))
do
    ip=${ACTUAL_IPS[$i]}
    ssh arch@$ip 'killall node-apollo node-synchs node-synchs-rr node-optsync client-optsync node-artemis client-artemis'
    ssh arch@$ip 'bash -ls --' < scripts/aws/throughput-vs-latency/$TYPE.sh $i $TESTDIR $DELAY $CLI_TYPE &
done