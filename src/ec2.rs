use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{
    DescribeInstancesError, DescribeInstancesRequest, Ec2, Ec2Client, Filter,
};

/// Use the link-local interface to get the instance ID
///
/// Reference:
/// docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html
pub fn get_instance_id() -> reqwest::Result<String> {
    let client = reqwest::Client::new();
    let url = "http://169.254.169.254/latest/meta-data/instance-id";
    let id = client.get(url).send()?.error_for_status()?.text()?;
    Ok(id)
}

#[derive(Debug)]
pub enum InstanceNameError {
    DescribeInstancesError(RusotoError<DescribeInstancesError>),
    MissingReservations,
    EmptyReservations,
    MissingInstances,
    EmptyInstances,
    MissingTags,
    MissingNameTag,
}

/// Use the instance ID to look up the instance's name tag
pub fn get_instance_name(
    instance_id: String,
) -> Result<String, InstanceNameError> {
    let client = Ec2Client::new(Region::default());
    let result = client
        .describe_instances(DescribeInstancesRequest {
            filters: Some(vec![Filter {
                name: Some("instance-id".to_string()),
                values: Some(vec![instance_id]),
            }]),
            ..Default::default()
        })
        .sync();
    match result {
        Ok(result) => {
            if let Some(reservations) = result.reservations {
                if let Some(reservation) = reservations.first() {
                    if let Some(instances) = &reservation.instances {
                        if let Some(instance) = instances.first() {
                            if let Some(tags) = &instance.tags {
                                for tag in tags.iter() {
                                    if tag.key == Some("Name".to_string()) {
                                        if let Some(value) = &tag.value {
                                            return Ok(value.clone());
                                        }
                                    }
                                }
                                return Err(InstanceNameError::MissingNameTag);
                            } else {
                                return Err(InstanceNameError::MissingTags);
                            }
                        } else {
                            return Err(InstanceNameError::EmptyInstances);
                        }
                    } else {
                        return Err(InstanceNameError::MissingInstances);
                    }
                } else {
                    return Err(InstanceNameError::EmptyReservations);
                }
            } else {
                return Err(InstanceNameError::MissingReservations);
            }
        }
        Err(err) => Err(InstanceNameError::DescribeInstancesError(err)),
    }
}
