trials=1000

while [[ $trials -gt 0 ]] ; do
    trials=$(( $trials - 1 ))
    W=80000 TESTDIR="testdata/b400-n3-p1024" bash scripts/synchs-test.sh &> run-log.log
done