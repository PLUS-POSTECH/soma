#!/bin/sh
export TERM=xterm
# TODO: Container internal port settings may be implemented afterwards
socat TCP-LISTEN:1024,pktinfo,reuseaddr,fork EXEC:"stdbuf -i 0 -o 0 {{ manifest.binary.entry }}",stderr
sleep infinity;
