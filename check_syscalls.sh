#!/bin/bash

# Fetch the FreeeBSD syscall table
raw_table=$(curl -sSL https://cgit.freebsd.org/src/plain/sys/kern/syscalls.master)

# Find syscalls implemented in the binary
syscalls=( $(grep -E "ACTION_\w+,$" pjdfstest.c | cut -d'_' -f2 | tr -d ',' | tr [A-Z] [a-z]) )

unsupported_syscalls=()
unknown_syscalls=()

for syscall in ${syscalls[@]}; do
  if ! grep -Rq "$syscall" "../nix/"; then
    if [[ "${raw_table}" == *"$syscall"* ]]; then
      unsupported_syscalls+=($syscall)
    else
      unknown_syscalls+=($syscall)
    fi
  fi
done

echo "Unsupported"
echo

for uns in ${unsupported_syscalls[@]}; do
  echo $uns
done

echo

echo "Unsupported & used"
echo
for uns in ${unsupported_syscalls[@]}; do
  if grep -Rq "$uns" tests; then
    echo $uns
  fi
done
echo

echo "Unknown syscalls"
echo
for unk in ${unknown_syscalls[@]}; do
  echo $unk
done
