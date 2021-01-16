# A script that exports the pdfs generated to the overleaf document after pdf
# cropping

PDFS=("scripts/throughput-vs-latency/vary-b/Tput-vs-latency-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-b/Tput-vs-w-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-b/Latency-vs-w-for-diff-b.pdf" "scripts/throughput-vs-latency/vary-p/Latency-vs-w-for-diff-p.pdf" "scripts/throughput-vs-latency/vary-p/Tput-vs-w-for-diff-p.pdf" "scripts/throughput-vs-latency/vary-p/Tput-vs-latency-for-diff-p.pdf" "scripts/throughput-vs-latency/vary-d/Delta-vs-Lat-for-diff-f.pdf" "scripts/throughput-vs-latency/vary-d/Delta-vs-Tput-for-diff-f.pdf" "scripts/throughput-vs-latency/vary-d/fault-vs-Lat-for-diff-f.pdf" "scripts/throughput-vs-latency/vary-d/fault-vs-Tput-for-diff-f.pdf")

MAY_BE_REMOVED=("scripts/ping/Ping-AWS-test.pdf")

for pdf in ${PDFS[@]}; do
    fname=`basename "$pdf" .pdf`
    pdfcrop "$pdf" /tmp/$fname.pdf
    cp /tmp/$fname.pdf ../Overleaf/Apollo/Figures/$fname.pdf
done

# for pdf in ${MAY_BE_REMOVED[@]}; do
#     if [ ! -e $pdf ]; then
#         continue
#     fi
#     fname=`basename $pdf .pdf`
#     pdfcrop "$pdf" /tmp/$fname.pdf
#     cp /tmp/$fname.pdf ../Overleaf/Apollo/Figures/$fname.pdf
# done