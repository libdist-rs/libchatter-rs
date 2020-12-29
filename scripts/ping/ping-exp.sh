killall ./target/release/ping &> /dev/null
killall ./target/release/echo &> /dev/null

./target/release/echo -p 10000 &> /dev/null &
sleep 1
./target/release/ping -m 1000 -s 127.0.0.1:10000 -c 1000000

killall ./target/release/ping &> /dev/null
killall ./target/release/echo &> /dev/null