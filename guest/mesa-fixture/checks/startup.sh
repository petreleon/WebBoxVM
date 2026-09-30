#!/bin/sh
set -u
status=0
/usr/bin/webboxvm-mesa-gles || status=$?
/usr/bin/webboxvm-mesa-vulkan || {
    result=$?
    test "$status" -ne 0 || status=$result
}
exit "$status"
