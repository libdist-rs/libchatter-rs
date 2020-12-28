# The script that manages the ping experiment on AWS

ssh -t arch@$1 'bash -ls' < scripts/aws/ping-exp.sh
