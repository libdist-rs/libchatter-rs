# Do the setup on the AWS Server

ssh -t arch@$1 'bash -ls' < scripts/aws/setup.sh