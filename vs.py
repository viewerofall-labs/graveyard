import os
import json
import sys
from pathlib import Path

def expand_path(path):
    if path.startswith("~"):
        return os.path.expanduser(path)
    return path

def list_directory(dir_path):
    expanded = expand_path(dir_path)

    try:
        entries = os.listdir(expanded)
    except Exception as e:
        return None, f"failed to read directory: {e}"

    files = []
    for entry in entries:
        full_path = os.path.join(expanded, entry)
        is_dir = os.path.isdir(full_path)

        files.append({
            "name": entry,
            "path": full_path,
            "isDir": is_dir
        })

    files.sort(key=lambda x: x["name"])
    return files, None

def main():
    if len(sys.argv) < 2:
        result = {
            "files": [],
            "error": "no directory path provided"
        }
        print(json.dumps(result))
        sys.exit(1)

    dir_path = sys.argv[1]
    files, err = list_directory(dir_path)

    if err:
        result = {
            "files": [],
            "error": err
        }
        print(json.dumps(result))
        sys.exit(1)

    result = {"files": files}
    print(json.dumps(result))

if __name__ == "__main__":
    main()
