#!/bin/sh

startfile="$1"
shift 1
outfile="$1"
shift 1
begin="$1"
shift 1
end="$1"
shift 1

# in case I want to pass the -y flag
force="$1"

ffmpeg $force -ss $begin -i "$startfile" -to $end "$outfile" && \
    ffplay -fast -nodisp -autoexit -loglevel -16 $outfile

if [ -f $outfile ]; then
    echo "Used $startfile: $begin to $end, to produce $outfile" \
        > "${outfile%%.*}".txt
    cat "${outfile%%.*}".txt
else
    echo "Usage: ./trimaudio.sh <file to trim> <output file> <start time> <end time>"
fi
