#!/bin/sh
set -e

{{ #each binary.file_entries }}
chmod {{ permissions }} "{{ target_path }}"
{{ /each }}

chown -R root:$PROB /home/$PROB
chmod 555 /.soma/start.sh
