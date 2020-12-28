# Do the setup on the AWS Server

for ip in "$@"
do
    ssh -t arch@$ip 'bash -ls' < scripts/aws/setup.sh &
done

wait