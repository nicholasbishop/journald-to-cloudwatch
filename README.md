# journald-to-cloudwatch

**This tool is no longer under active development. If you are interested in taking over or repurposing the name on crates.io, feel free to contact me: nbishop@nbishop.net**

This is a simple service that copies logs from journald to AWS
CloudWatch Logs.

The implementation is very basic. It does not copy logs that were
created prior to journald-to-cloudwatch starting. It has only one
configuration option, which is the name of the log group. The log
stream name is derived from the instance name (the service assumes it
is running on an EC2 instance).

## Usage

To build the service for EC2:

    tools/package.py
    
This builds in a Docker container that has the libraries an awslinux2
EC2 instance would have.

The output is `release/journald-to-cloudwatch-{version}.tar.gz`. Copy
that to an EC2 instance. There is an example service configuration
file in the tarball. Copy that to `/etc/systemd/system/` and modify
`LOG_GROUP_NAME` to the name of your log group. Note that the log
group must exist for the service to work; it will not create the log
group.

## IAM policy

The following permissions are required:

    logs:CreateLogStream
    logs:DescribeLogStreams
    logs:PutLogEvents
