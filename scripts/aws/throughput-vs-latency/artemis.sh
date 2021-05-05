ID=$1
TESTDIR=${2:-"testdata/b100-n3"}
DELAY=${3:-"50"}
CLI_TYPE="-s"

cd libchatter-rs

# sleep 30
# echo "Using arguments: --config $TESTDIR/nodes-$ID.json --ip ips_file --delta "$DELAY" -s"

./target/release/node-artemis \
    --config $TESTDIR/nodes-$ID.json \
    --ip ips_file \
    --delta "$DELAY" \
    $CLI_TYPE &