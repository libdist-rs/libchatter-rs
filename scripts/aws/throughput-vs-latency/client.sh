TESTDIR=${1:-"testdata/b100-n3"}
W=${2:-"50000"}
CLI_TYPE=${3:-"client-apollo"}

cd libchatter-rs

# sleep 30

./target/release/$CLI_TYPE \
    --config $TESTDIR/client.json \
    -i cli_ip_file \
    -w $W \
    -m 1000000
