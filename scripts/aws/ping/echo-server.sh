cd libchatter-rs

killall ./target/release/echo &> /dev/null

./target/release/echo -p 10000 &> /dev/null &