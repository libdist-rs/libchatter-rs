ID=$1
TESTDIR=${2:-"testdata/b100-n3"}
DELAY=${3:-"50"}

cd libchatter-rs

echo "Using arguments: --config $TESTDIR/nodes-$ID.json --ip ips_file --delta "$DELAY" -s"

./target/release/node-apollo \
    --config $TESTDIR/nodes-$ID.json \
    --ip ips_file \
    --delta "$DELAY" \
    -s &