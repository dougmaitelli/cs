"""
Helper for running cs in a pseudo-terminal to simulate interactive user input.

Usage: python3 pty_helper.py <cs_binary> <config_home> <extra_env_k=v,...> arg1 arg2 ...

Stdin: JSON of {"keystrokes": [[bytes as ints]]}
Stdout: Clean output text
Stderr: "EXIT:<code>"
"""
import sys
import os
import json
import pty
import subprocess
import time
import re
import select

def main():
    args = sys.argv[1:]
    cs_bin = args[0]
    config_home = args[1]
    extra_env_str = args[2] if len(args) > 2 else ""
    cs_args = args[3:]

    # Parse extra env vars
    extra_env = {}
    if extra_env_str:
        for pair in extra_env_str.split(","):
            if "=" in pair:
                k, v = pair.split("=", 1)
                extra_env[k] = v

    env = dict(os.environ)
    env["XDG_CONFIG_HOME"] = config_home
    env["SHELL"] = "/bin/bash"
    for k, v in extra_env.items():
        env[k] = v

    cmd = [cs_bin] + cs_args

    master_fd, slave_fd = pty.openpty()

    proc = subprocess.Popen(
        cmd,
        stdin=slave_fd,
        stdout=slave_fd,
        stderr=slave_fd,
        env=env,
    )

    os.close(slave_fd)

    # Read keystrokes from stdin
    stdin_data = sys.stdin.read()
    data = json.loads(stdin_data)
    keystrokes = []
    for k in data["keystrokes"]:
        if isinstance(k, list):
            keystrokes.append(bytes(k))
        elif isinstance(k, str):
            keystrokes.append(k.encode("utf-8"))
        else:
            keystrokes.append(bytes(k))

    def send_input():
        time.sleep(2.0)
        for raw in keystrokes:
            try:
                os.write(master_fd, raw)
            except OSError:
                pass
            time.sleep(0.5)
        time.sleep(3.0)
        try:
            os.close(master_fd)
        except:
            pass

    import threading
    t = threading.Thread(target=send_input)
    t.start()

    def read_all(timeout_secs=15):
        output = b""
        start = time.time()
        while time.time() - start < timeout_secs:
            if proc.poll() is not None:
                break
            try:
                ready, _, _ = select.select([master_fd], [], [], 0.1)
                if ready:
                    data = os.read(master_fd, 4096)
                    output += data
            except OSError:
                break
        try:
            data = os.read(master_fd, 4096)
            output += data
        except OSError:
            pass
        return output

    try:
        output = read_all()
    except OSError:
        output = b""

    proc.wait()
    t.join(timeout=5)

    clean = output.decode("utf-8", errors="replace")
    # Remove ANSI escape codes
    clean = re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]", "", clean)
    clean = re.sub(r"\x1b\[[0-9]*K", "", clean)
    clean = re.sub(r"\x1b\[\?25[lh]", "", clean)
    # Remove duplicate blank lines from cursor clears
    lines = clean.split("\n")
    result_lines = []
    for line in lines:
        if result_lines and result_lines[-1] == "" and line == "":
            continue
        result_lines.append(line)
    clean = "\n".join(result_lines).rstrip()

    print(clean, end="")
    print()
    sys.stderr.write("EXIT:" + str(proc.returncode) + "\n")
    sys.stderr.flush()

if __name__ == "__main__":
    main()
