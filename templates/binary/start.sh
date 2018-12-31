#!/bin/sh
export TERM=xterm
socat TCP-LISTEN:1024,pktinfo,reuseaddr,fork EXEC:"stdbuf -i 0 -o 0 {{ repository.manifest.binary.entry }}",stderr
sleep infinity;
