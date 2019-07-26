#!/usr/bin/env python3

import os
import subprocess
import sys

try:
    import toml
except ModuleNotFoundError:
    print('missing toml package; try "pip3 install --user toml"')
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
REPO_DIR = os.path.join(SCRIPT_DIR, os.pardir)

def run_cmd(*cmd):
    print(' '.join(cmd))
    subprocess.check_call(cmd)


def read_version():
    path = os.path.join(REPO_DIR, 'Cargo.toml')
    with open(path) as rfile:
        cargo = toml.load(rfile)
        return cargo['package']['version']


def main():
    version = read_version()
    image_name = 'jtc-image'
    dockerfile = os.path.join(REPO_DIR, 'tools/Dockerfile')
    output_dir = os.path.join(REPO_DIR, "release")
    exe_path = os.path.join(output_dir, 'journald-to-cloudwatch')
    tar_path = os.path.join(
        output_dir, 'journald-to-cloudwatch-{}.tar.gz'.format(version))

    run_cmd('sudo', 'docker', 'build', '-t', image_name, '-f', dockerfile, '.')
    run_cmd('sudo', 'docker', 'run',
            '-e', 'CARGO_HOME=/cargo',
            '-e', 'CARGO_TARGET_DIR=/cache',
            '-v', 'jtc-cargo-volume:/cargo',
            '-v', 'jtc-cache-volume:/cache',
            '-v', '{}:/host:z'.format(output_dir),
            image_name)
    run_cmd('sudo', 'chown', '{}:{}'.format(os.getuid(), os.getgid()),
            exe_path)
    if os.path.exists(tar_path):
        os.remove(tar_path)
    run_cmd('tar', 'czf', tar_path, '-C', output_dir,
            os.path.basename(exe_path),
            'journald-to-cloudwatch.service')


if __name__ == '__main__':
    main()
