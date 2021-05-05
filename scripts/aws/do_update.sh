# Do the setup on the AWS Server

FILE="${1:-/dev/stdin}"
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

for ip in "${IPS[@]}"
do
    ssh -t arch@$ip 'bash -ls' < scripts/aws/update.sh &
done

wait