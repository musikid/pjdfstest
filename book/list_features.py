#!/usr/bin/env python3

from subprocess import run


def main():
    proc = run(["../../rust/target/debug/pjdfstest", "--list-features"], check=True, capture_output=True)
    proc.check_returncode()
    for line in proc.stdout.decode().splitlines():
        feature_name, desc = line.split(": ", 1)
        print(f"- **{feature_name}** - {desc}")

if __name__ == "__main__":
    main()