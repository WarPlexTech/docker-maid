use bollard::Docker;
use bollard::errors::Error;
use bollard::models::{ContainerSummary};
use bollard::query_parameters::{CreateImageOptionsBuilder, ListContainersOptionsBuilder};
use futures::StreamExt;
use log::error;

/// Helper function to connect to the local docker socket
pub fn connect_to_docker() -> Docker {
    Docker::connect_with_socket_defaults().unwrap_or_else(|e| {
        error!("Unable to connect to docker socket. Is /var/run/docker.sock mounted?");
        panic!("{}", e);
    })
}

/// Helper function to fetch all containers from docker
pub async fn get_all_containers(docker: &Docker) -> Result<Vec<ContainerSummary>, Error> {
    let options = ListContainersOptionsBuilder::new()
        .all(true)
        .build();

    docker.list_containers(Some(options)).await
}

/// Helper function to pull an image from a docker registry
pub async fn pull_image(docker: &Docker, image_name: &str) -> Result<(), Error> {
    let full_image_name = if image_name.contains(':') {
        image_name.to_string()
    } else {
        format!("{}:latest", image_name)
    };

    let options = CreateImageOptionsBuilder::new()
        .from_image(&full_image_name)
        .build();

    let mut stream = docker.create_image(Some(options), None, None);
    while let Some(result) = stream.next().await {
        match result {
            Ok(_) => (),
            Err(e) => return Err(e)
        }
    }

    Ok(())
}