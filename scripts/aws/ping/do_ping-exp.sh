# The script that manages the ping experiment on AWS
ECHO_SERVER=$1
PING_CLIENT=$2
ECHO_SERVER_PVT_IP=$3
OUTPUT=${3:-"scripts/aws/ping/ping-raw.log"}

if [ $# -ne 3 ];then
    echo "Incorrect number of arguments"
    echo "Usage: $0 <Echo Server IP> <Ping Client IP> <Pvt. IP of the Echo Server>"
    exit 1
fi

ssh -t arch@$ECHO_SERVER 'bash -ls' < scripts/aws/echo-server.sh
sleep 1
ssh -t arch@$PING_CLIENT 'bash -ls --' < scripts/aws/ping-client.sh "$ECHO_SERVER_PVT_IP:10000" &> $OUTPUT
ssh -t arch@$ECHO_SERVER 'killall "./target/release/echo" &> /dev/null'
