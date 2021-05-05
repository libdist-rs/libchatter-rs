cd libchatter-rs

ID=${1:-"0"}
TESTDIR="testdata/b100-n3"
DELAY=50
CLI_TYPE="-s"

./target/debug/node-artemis \
    --config $TESTDIR/nodes-$ID.json \
    --ip ips_file \
    --delta "$DELAY" \
    $CLI_TYPE -v &