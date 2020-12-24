# A script to test quickly

killall node-apollo

TESTDIR=${TESTDIR:="testdata/b100-n3"}
TYPE=${TYPE:="release"}

./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-0.json \
    --ip ip_file \
    -s &
./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-1.json \
    --ip ip_file \
    -s &
./target/$TYPE/node-apollo \
    --config $TESTDIR/nodes-2.json \
    --ip ip_file \
    -s &

sleep 5
# Nodes must be ready by now
./target/$TYPE/client-apollo \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w 1200 \
    -m 1000000

# Client has finished; Kill the nodes
killall ./target/$TYPE/node-apollo