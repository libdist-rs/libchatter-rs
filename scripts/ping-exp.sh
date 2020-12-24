source scripts/root.sh

killall ./target/release/ping &> /dev/null
killall ./target/release/echo &> /dev/null

./target/release/echo -p 10000 &> /dev/null &
sleep 1
./target/release/ping -c 100000 -m 1000 -s 127.0.0.1:10000 -i 1000 -t 10 | cut -d":" -f2 | sed 's/ //g'

killall ./target/release/ping &> /dev/null
killall ./target/release/echo &> /dev/null