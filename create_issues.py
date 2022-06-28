#!/usr/bin/env python

import json
import subprocess
from time import sleep
import re

tree = json.loads(subprocess.check_output(["tree", "-J", "tests/"], text=True))[0]

files = {folder["name"]: [file["name"] for file in folder["contents"]] for folder in tree["contents"] if folder["type"] == "directory"}

issue_template = """# {syscall}

This issue tracks the progress of the rewrite for the {syscall} folder.

{task_list}
"""

milestone = "Rewrite tests"

for syscall, files in files.items():
    task_list = []
    for file in files:
        path = f"tests/{syscall}/{file}"
        header = open(path).read(600)
        match = re.search('desc="(.*?)"', header)
        task_list.append(f"- [ ] {file} | {match.group(1) if match else 'Unknown description'}")
    title = f"Rewrite {syscall} tests"
    labels = "rewrite"
    body = issue_template.format(syscall=syscall, task_list="\n".join(task_list))
    cmd = ["gh", "issue", "create", 
        "--title", title, 
        "--body", body,
        "--label", labels,
        "--milestone", milestone]
    print(subprocess.check_output(cmd, text=True))
    sleep(10)
