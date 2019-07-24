#!/usr/bin/env python3

import os
import subprocess

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
REPO_DIR = os.path.join(SCRIPT_DIR, os.pardir)

def run_cmd(*cmd):
    print(' '.join(cmd))
    subprocess.check_call(cmd)


def main():
    image_name = 'jtc-image'
    dockerfile = os.path.join(REPO_DIR, 'tools/Dockerfile')
    output_dir = os.path.join(REPO_DIR, "release")
    exe_path = os.path.join(output_dir, 'journald-to-cloudwatch')
    tar_path = os.path.join(output_dir, 'journald-to-cloudwatch.tar.gz')

    run_cmd('sudo', 'docker', 'build', '-t', image_name, '-f', dockerfile, '.')
    run_cmd('sudo', 'docker', 'run',
            '-v', 'jtc-cargo-git-volume:/home/rust/.cargo/git',
            '-v', 'jtc-cargo-reg-volume:/home/rust/.cargo/registry',
            '-v', 'jtc-cargo-tgt-volume:/home/rust/src/target',
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
