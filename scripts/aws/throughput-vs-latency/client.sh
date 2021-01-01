TESTDIR=${1:-"testdata/b100-n3"}
W=${2:-"50000"}
TYPE=${3:-"apollo"}

cd libchatter-rs

./target/release/client-$TYPE \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000
