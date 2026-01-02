use std::collections::HashMap;
use anyhow::{Context, Result};
use bollard::Docker;
use bollard::errors::Error;
use bollard::models::{ContainerSummary};
use bollard::query_parameters::{CreateImageOptionsBuilder, ListContainersOptionsBuilder};
use futures::StreamExt;
use log::{error, info};

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

/// Helper function to fetch a specific container summary from docker
pub async fn get_container_summary(docker: &Docker, container_id: &str) -> Result<Vec<ContainerSummary>, Error> {
    let options = ListContainersOptionsBuilder::new()
        .filters(&HashMap::from([(
            "id",
            vec![container_id.to_string()]
        )]))
        .build();

    docker.list_containers(Some(options)).await
}

/// Helper function to check if a new digest is available for a container's image
pub async fn is_newer_digest_available(docker: &Docker, container: &ContainerSummary) -> Result<bool> {
    let current_image_name = container.image.as_deref()
        .context("Container image name not found")?;

    let current_digest = container.image_id.as_deref()
        .context("Container has no image ID")?;

    let inspect_image_response = docker.inspect_image(&current_image_name).await
        .with_context(|| format!("Failed to inspect container image `{}`", current_image_name))?;

    let repo_digests = inspect_image_response.repo_digests.unwrap_or_default();
    if repo_digests.is_empty() {
        info!(
            "\t\t-> Image `{}` does not have any repo digests. Update skipped.",
            &current_image_name
        );
        return Ok(false);
    }

    pull_image(&docker, &current_image_name).await
        .with_context(|| format!("Failed to pull image `{}`", current_image_name))?;

    let latest_image = docker.inspect_image(&current_image_name).await
        .context("Failed to image after pull")?;

    let latest_digest = latest_image.id.context("Latest image has no ID")?;

    Ok(latest_digest != current_digest)
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