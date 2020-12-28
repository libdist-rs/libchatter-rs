# A script that exports the pdfs generated to the overleaf document after pdf
# cropping

PDFS=("scripts/ping/Ping-AWS.pdf" "scripts/throughput-vs-latency/vary-b/Tput-vs-latency-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-b/Tput-vs-w-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-b/Latency-vs-w-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-p/Latency-vs-w-for-diff-p.pdf" "scripts/throughput-vs-latency/vary-p/Tput-vs-w-for-diff-p.pdf" "scripts/throughput-vs-latency/vary-p/Tput-vs-latency-for-diff-p.pdf")

for pdf in ${PDFS[@]}; do
    fname=`basename "$pdf" .pdf`
    pdfcrop "$pdf" /tmp/$fname.pdf
    cp /tmp/$fname.pdf ../Overleaf/Apollo/Figures/$fname.pdf
done