# Do the setup on the AWS Server

for ip in "$@"
do
    ssh -t arch@$ip 'rm -rf ~/.cargo/registry/index/*.'
    ssh -t arch@$ip 'rm -rf ~/.cargo/.package-cache'
    ssh -t arch@$ip 'sudo rm -rf /var/lib/pacman/db.lck'
    ssh -t arch@$ip 'rm -rf libchatter-rs/target'
done

wait