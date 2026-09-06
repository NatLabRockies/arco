#!/usr/bin/env python3
import json
import os
import shutil
import sys
from pathlib import Path

args = sys.argv[1:]
with Path(os.environ["FAKE_GH_LOG"]).open("a") as log:
    log.write(json.dumps(args) + "\n")
response_path = Path(os.environ["FAKE_GH_RESPONSES"])
responses = json.loads(response_path.read_text())
key = json.dumps(args, separators=(",", ":"))
values = responses.get(key)
if values is None:
    print(f"unexpected gh call: {args}", file=sys.stderr)
    raise SystemExit(2)
if (
    isinstance(values, list)
    and values
    and isinstance(values[0], dict)
    and "_response" in values[0]
):
    response = values.pop(0)["_response"]
    responses[key] = values
    response_path.write_text(json.dumps(responses))
else:
    response = values
if args[:2] == ["release", "download"]:
    destination = Path(args[args.index("--dir") + 1])
    destination.mkdir(parents=True, exist_ok=True)
    source = Path(os.environ["FAKE_GH_DOWNLOADS"])
    for index, arg in enumerate(args):
        if arg == "--pattern":
            shutil.copy2(source / args[index + 1], destination)
print(json.dumps(response) if not isinstance(response, str) else response)
