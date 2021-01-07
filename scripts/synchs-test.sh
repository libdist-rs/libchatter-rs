# A script to test quickly

killall node-synchs &> /dev/null

TESTDIR=${TESTDIR:="testdata/b100-n3"}
TYPE=${TYPE:="release"}
W=${W:="100000"}

./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-0.json \
    --delta 50 \
    --ip ip_file &
./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-1.json \
    --delta 50 \
    --ip ip_file &
./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-2.json \
    --delta 50 \
    --ip ip_file &

sleep 20
# Nodes must be ready by now
./target/$TYPE/client-synchs \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000

# Client has finished; Kill the nodes
killall ./target/$TYPE/node-synchs &> /dev/null