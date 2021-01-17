# A script to test quickly

killall node-apollo &> /dev/null

TESTDIR=${TESTDIR:="testdata/b100-n3"}
TYPE=${TYPE:="release"}
W=${W:="80000"}

./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-0.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 0.log &
./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-1.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 1.log &
./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-2.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 2.log &

sleep 60
# Nodes must be ready by now
./target/$TYPE/client-apollo \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000 $1

# Client has finished; Kill the nodes
killall ./target/$TYPE/node-apollo &> /dev/null