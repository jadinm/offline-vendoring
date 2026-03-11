#!/bin/bash

set -eux -o pipefail -o noclobber

sudo bash -c 'echo "127.0.0.1 crates.io" >> /etc/hosts'
sudo bash -c 'echo "127.0.0.1 pypi.org" >> /etc/hosts'
# Check that the websites are blocked
! curl crates.io -f -s
! curl pypi.org -f -s
