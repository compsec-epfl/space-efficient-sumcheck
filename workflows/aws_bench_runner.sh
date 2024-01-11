#!/bin/sh

# Function to handle errors
handle_error() {
    echo "An error occurred. Exiting." >&2
    if [ -n "$instance_id" ]; then
        aws ec2 terminate-instances --instance-ids $instance_id
    fi
    exit 1
}

# Set up trap to call handle_error function on ERR signal
trap 'handle_error' ERR

# Define your AWS credentials and region (you can set these manually if you're working locally)
export AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export AWS_DEFAULT_REGION="eu-central-1"

# Define some other stuff (you can set these manually if you're working locally)
export GITHUB_REPOSITORY="$GITHUB_REPOSITORY"
export GITHUB_SHA="$GITHUB_SHA"
export GITHUB_RUN_NUMBER="$GITHUB_RUN_NUMBER"
export GITHUB_REF="$GITHUB_REF"
export READ_GITHUB_ACCESS_TOKEN="$READ_GITHUB_ACCESS_TOKEN"

echo $AWS_ACCESS_KEY_ID

# Create an EC2 instance
instance_id=$(aws ec2 run-instances \
    --launch-template LaunchTemplateId=lt-05edf3d1d43301f02 \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=bench-$GITHUB_REF}, {Key=Group,Value=bench-machines}]" \
    --query 'Instances[0].InstanceId' \
    --output text)

echo "EC2 instance with ID $instance_id is being created."

# Wait for the instance to be running before proceeding
aws ec2 wait instance-running --instance-ids $instance_id

echo "EC2 instance is now running."

# # Get the public IP address of the instance
public_ip=$(aws ec2 describe-instances --instance-ids $instance_id --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)

# # Run a command on the EC2 instance and pipe the output to an S3 bucket
ssh -i ~/Desktop/bench-epfl.pem -o StrictHostKeyChecking=no ec2-user@$public_ip "export GITHUB_ACCESS_TOKEN=$GITHUB_ACCESS_TOKEN && sh -s" < ./setup_bench_machine.sh


# Terminate the EC2 instance
aws ec2 terminate-instances --instance-ids $instance_id
