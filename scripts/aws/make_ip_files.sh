PVT_IP_FILE=${1:-"scripts/aws/pvt_ips.log"}
IPS_FILE=${2:-"scripts/aws/ips_file.log"}
CLI_IPS_FILE=${3:-"scripts/aws/cli_ips.log"}
PVT_IPS=()
IPS=()
BASE_PORT=4000
CLIENT_PORT=10000

if [ -e "$IPS_FILE" ]; then
    echo "File [$IPS_FILE] already exists"
    rm -rf $IPS_FILE
fi

if [ -e "$CLI_IPS_FILE" ]; then
    echo "File [$CLI_IPS_FILE] already exists"
    rm -rf $CLI_IPS_FILE
fi

idx=0

while IFS= read -r line; do
    IPS+=($line":"$(($BASE_PORT+$idx)))
    PVT_IPS+=($line":"$(($CLIENT_PORT+$idx)))
    idx=$(($idx+1))
done < $PVT_IP_FILE

unset IPS[-1]
unset PVT_IPS[-1]

for i in ${IPS[@]}; do
    echo "$i" >> $IPS_FILE
done

for i in ${PVT_IPS[@]}; do
    echo "$i" >> $CLI_IPS_FILE
done