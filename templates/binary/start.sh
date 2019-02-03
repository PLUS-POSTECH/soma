#!/bin/sh
export TERM=xterm
# TODO: Container internal port settings may be implemented afterwards
socat tcp-listen:1337,pktinfo,reuseaddr,fork exec:"{{ binary.cmd }}",pty,ctty,raw,echo=0,stderr
sleep infinity;
