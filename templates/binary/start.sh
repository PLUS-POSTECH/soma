#!/bin/sh
export TERM=xterm
socat TCP-LISTEN:{{ port }},pktinfo,reuseaddr,fork EXEC:"stdbuf -i 0 -o 0 {{ entry }}",stderr
sleep infinity;
