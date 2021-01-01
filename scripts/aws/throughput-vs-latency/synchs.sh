ID=$1
TESTDIR=${2:-"testdata/b100-n3"}

cd libchatter-rs

./target/release/node-synchs \
    --config $TESTDIR/nodes-$ID.json \
    --ip ips_file &