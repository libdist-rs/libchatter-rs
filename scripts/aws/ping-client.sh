cd libchatter-rs

M=${2:-"1000"}
C=${3:="100000"}

killall ./target/release/echo &> /dev/null

./target/release/ping -c "$C" -m "$M" -s $1 -i 1000 -t 10

killall ./target/release/ping &> /dev/null