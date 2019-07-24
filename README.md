# journald-to-cloudwatch

This is a simple service that copies logs from journald to AWS
CloudWatch Logs.

The implementation is very basic. It does not copy logs that were
created prior to journald-to-cloudwatch starting. It does not batch
logs. It has only one configuration option, which is the name of the
log group. The log stream name is derived from the instance name (the
service assumes it is running on an EC2 instance).

## Usage

First, build the service:

    cargo build --release
    
The output is `target/release/journald-to-cloudwatch`. Copy that to an
EC2 instance. There is an example service configuration file in the
repo, `journald-to-cloudwatch.service`. Copy that to the instance
under `/etc/systemd/system/` and modify `LOG_GROUP_NAME` to the name
of your log group. Note that the log group must exist for the service
to work; it will not create the log group.

## IAM policy

The following permissions are required:

    logs:CreateLogStream
    logs:DescribeLogStreams
    logs:PutLogEvents
