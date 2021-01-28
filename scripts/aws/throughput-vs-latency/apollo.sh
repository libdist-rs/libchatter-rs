ID=$1
TESTDIR=${2:-"testdata/b100-n3"}
DELAY=${3:-"50"}
CLI_TYPE=${4:-"client-apollo"}

cd libchatter-rs

if [ $CLI_TYPE == "client-apollo" ]; then
    CLI_TYPE="-s"
else
    CLI_TYPE=""
fi

# sleep 30
# echo "Using arguments: --config $TESTDIR/nodes-$ID.json --ip ips_file --delta "$DELAY" -s"

./target/release/node-apollo \
    --config $TESTDIR/nodes-$ID.json \
    --ip ips_file \
    --delta "$DELAY" \
    --sleep 120 \
    $CLI_TYPE &