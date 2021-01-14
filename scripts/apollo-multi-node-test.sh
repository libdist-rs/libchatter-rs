# A script to test quickly

killall node-apollo &> /dev/null

TESTDIR=${TESTDIR:="testdata/test"}
TYPE=${TYPE:="release"}
W=${W:="80000"}

for((i=0;i<7;i++)); do
./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-$i.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 > $i.log &
done

sleep 20
# Nodes must be ready by now
./target/$TYPE/client-apollo \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000 $1

# Client has finished; Kill the nodes
killall ./target/$TYPE/node-apollo &> /dev/null