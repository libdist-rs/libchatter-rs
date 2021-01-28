FILE="${1:-/dev/stdin}"
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

for ip in "${IPS[@]}"
do
    ssh -o StrictHostKeyChecking=accept-new arch@$ip 'ip address show' | \
    grep "inet .* brd" | \
    sed 's/ brd.*//g' | \
    sed 's/inet //' | \
    sed 's;/.*;;g' | \
    sed 's/.* //g'
done