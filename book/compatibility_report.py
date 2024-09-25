#!/usr/bin/env python3

from dataclasses import dataclass
from os import path, walk
from typing import Iterable


def get_tests_filenames_from_spec(tests_list: str):
    with open(tests_list, "r") as file:
        for line in file:
            yield tuple(line.split("\t"))


def get_rust_files(rust_dir: str):
    for root, _, files in walk(rust_dir):
        for file in files:
            if file.endswith(".rs"):
                yield path.join(root, file)


def search_string_in_file(file_path: str, string: str):
    with open(file_path, "r") as file:
        for line in file:
            if string in line:
                return True
    return False


def get_tests_with_and_without_rust_files(rust_dir: str, tests_files: Iterable[str]):
    rust_files = set(get_rust_files(rust_dir))
    tests_with_rust_files = set()
    tests_without_rust_files = set()
    for test in tests_files:
        if any(search_string_in_file(rust_file, test) for rust_file in rust_files):
            tests_with_rust_files.add(test)
        else:
            tests_without_rust_files.add(test)

    return tests_with_rust_files, tests_without_rust_files


def find_file_description(file_path: str):
    with open(file_path, "r") as file:
        for line in file:
            if line.startswith("desc="):
                return line[len('desc="') : -2].strip()
    return ""


@dataclass
class Test:
    name: str
    has_rust_file: bool
    description: str


def main():
    rust_dir = "../../rust/src/tests"
    tests_dir = "../old_testcases.tsv"
    tests_files = set(get_tests_filenames_from_spec(tests_dir))
    tests_with_rust_files, _ = (
        get_tests_with_and_without_rust_files(rust_dir, map(lambda v: v[0], tests_files))
    )
    syscalls: dict[str, list[Test]] = {}
    for testfile, desc in tests_files:
        syscall, test_name = testfile.split("/")
        if syscall not in syscalls:
            syscalls[syscall] = []

        syscalls[syscall].append(
            Test(
                test_name,
                testfile in tests_with_rust_files,
                desc,
            )
        )

    for syscall, tests in sorted(syscalls.items()):
        tests.sort(key=lambda t: t.name)
        print("<details>")
        print(
            f"<summary>{syscall} <progress value='{sum(1 for t in tests if t.has_rust_file)}' "
            f"max='{len(tests)}'></progress></summary>"
        )
        print("<table>")
        print("<tr>")
        print("<th>Test</th>")
        print("<th>Description</th>")
        print("<th>Converted</th>")
        print("</tr>")
        for test in tests:
            print("<tr>")
            print(f"<td>{test.name}</td>")
            print(f"<td>{test.description}</td>")
            print(f"<td>{'Yes' if test.has_rust_file else 'No'}</td>")
            print("</tr>")
        print("</table>")
        print("</details>")
        print()


if __name__ == "__main__":
    main()
