# A script to test quickly

killall node-synchs &> /dev/null

TESTDIR=${TESTDIR:="testdata/b100-n3"}
TYPE=${TYPE:="release"}
W=${W:="3000"}

./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-0.json \
    --ip ip_file &
./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-1.json \
    --ip ip_file &
./target/$TYPE/node-synchs \
    --config $TESTDIR/nodes-2.json \
    --ip ip_file &

sleep 5
# Nodes must be ready by now
./target/$TYPE/client-synchs \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000

# Client has finished; Kill the nodes
killall ./target/$TYPE/node-synchs &> /dev/null