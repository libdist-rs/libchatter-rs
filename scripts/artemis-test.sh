# A script to test quickly

set -e
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

make artemis
make artemis-release

TESTDIR=${TESTDIR:="testdata/b100-n3"}
TYPE=${TYPE:="release"}
W=${W:="80000"}

./target/$TYPE/node-artemis \
    --config $TESTDIR/nodes-0.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 0.log &
./target/$TYPE/node-artemis \
    --config $TESTDIR/nodes-1.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 1.log &
./target/$TYPE/node-artemis \
    --config $TESTDIR/nodes-2.json \
    --ip ip_file \
    --sleep 20 \
    -s $1 &> 2.log &

sleep 15
# Nodes must be ready by now
./target/$TYPE/client-artemis \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000 $1 &> client.log &

wait