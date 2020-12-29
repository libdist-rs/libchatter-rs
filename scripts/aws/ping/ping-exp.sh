# The script that manages the ping experiment on AWS

SERVERS_FILE=$1
PVT_IPS_FILE=$2
IPS=()

if [ $# -lt 2 ];then
    echo "Incorrect number of arguments"
    echo "Usage: $0 <Public IPS> <Pvt. IPs> [Output file]"
    exit 1
fi

while IFS= read -r line; do
  IPS+=($line)
done < $SERVERS_FILE

ECHO_SERVER=${IPS[0]}
PING_CLIENT=${IPS[1]}

IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $PVT_IPS_FILE

PVT_IP=${IPS[0]}

OUTPUT=${3:-"scripts/aws/ping/ping-raw.log"}

MSGS=(100 1000 10000 100000)

for m in ${MSGS[@]} ;do
  ssh arch@$ECHO_SERVER 'bash -ls' < scripts/aws/ping/echo-server.sh
  sleep 10
  echo "DP[Message]:$m" >> $OUTPUT
  ssh arch@$PING_CLIENT 'bash -ls --' < scripts/aws/ping/ping-client.sh "$PVT_IP:10000" "$m" &>> $OUTPUT
  ssh arch@$ECHO_SERVER 'killall "echo"'
  sleep 10
done