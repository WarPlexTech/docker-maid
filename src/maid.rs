use crate::enums::{ImagesPruneMode, ContainersUpdateMode};
use crate::utils::{connect_to_docker, get_all_containers, pull_image};
use bollard::Docker;
use bollard::models::{ContainerCreateBody, ContainerSummaryStateEnum};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, InspectContainerOptionsBuilder, PruneImagesOptionsBuilder,
    RemoveContainerOptionsBuilder, StartContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use log::{error, info, warn};
use std::collections::HashMap;

pub async fn housekeeping() {
    info!("Housekeeping duties underway.");

    // Read environment variables
    let update_mode = ContainersUpdateMode::from_env();
    let prune_mode = ImagesPruneMode::from_env();

    // Connect to docker socket
    let docker = connect_to_docker();

    // Update containers if UpdateMode is not None
    if update_mode != ContainersUpdateMode::None {
        update_images(&update_mode, &docker).await;
    }

    // Prune images if PruneMode is not None
    if prune_mode != ImagesPruneMode::None {
        prune_images(&prune_mode, &docker).await;
    }
}

/// Duty: Checks for new container image digests and updates containers or notifies the user based on the `update_mode` setting.
async fn update_images(update_mode: &ContainersUpdateMode, docker: &Docker) {
    // Updating tags to their latest digests
    info!("[DUTY] Checking for new container image digests...");

    // Fetch containers list
    let containers = match get_all_containers(&docker).await {
        Ok(containers) => containers,
        Err(e) => {
            error!(
                "\t-> Failed to fetch containers list, will retry on the next housekeeping round. (Internal error: `{}`).",
                e
            );
            return;
        }
    };

    info!("\t-> Found `{}` containers.", containers.len());
    info!("\t-> Processing the containers in `{}` mode", update_mode);
    for container in containers {
        // Init
        let current_container_id = match container.id.as_deref() {
            Some(id) => id,
            None => {
                error!("\t-> Failed to fetch container ID. Update skipped.");
                continue;
            }
        };

        let current_container_name = match container.names.as_deref() {
            Some(id) => id.concat(),
            None => {
                error!("\t-> Failed to fetch container name. Update skipped.");
                continue;
            }
        };

        let container_state = match container.state.as_ref() {
            Some(state) => state,
            None => {
                error!(
                    "\t-> Container `{}` has no state information. Update skipped.",
                    current_container_name
                );
                continue;
            }
        };

        // Skip containers without image information
        let image_name = match container.image.as_ref() {
            Some(image) => image,
            None => {
                warn!(
                    "\t\t-> Container `{}` has no image information. Update skipped.",
                    container.names.unwrap_or_default().concat()
                );
                continue;
            }
        };

        info!("\t-> Checking container `{}`.", current_container_name);

        let current_digest = match container.image_id.as_ref() {
            Some(digest) => digest,
            None => {
                error!(
                    "\t\t-> Container `{}` has no image ID. Update skipped.",
                    current_container_name
                );
                continue;
            }
        };

        match pull_image(&docker, &image_name).await {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "\t\t-> Failed to pull image `{}`. (Internal error: `{}`). Update skipped.",
                    image_name, e
                );
                continue;
            }
        };

        let latest_digest = match docker.inspect_image(&image_name).await {
            Ok(image) => image.id.unwrap_or_default(),
            Err(e) => {
                error!(
                    "\t\t-> Failed to inspect image `{}`. (Internal error: `{}`). Update skipped.",
                    image_name, e
                );
                continue;
            }
        };

        // If the image digest is unchanged, skip update
        if &latest_digest == current_digest && false {
            info!("\t\t-> Container is up to date.");
            continue;
        }

        // Since we already pulled the latest digest to compare with the one used by the container,
        // we can safely update the container by restarting it.
        info!("\t\t-> New digest found for image `{}`", image_name);

        if update_mode == &ContainersUpdateMode::Update {
            info!(
                "\t\t\t-> Container `{}` will be recreated.",
                current_container_name
            );

            match update_container(&docker, current_container_id, &current_container_name, container_state).await {
                Ok(_) => (),
                Err(e) => {
                    error!("\t\t\t-> {}. Update skipped.", e);
                    continue;
                }
            };
        } else {
            warn!("\t\t\t-> Container update not set to `Update`, skipping.");
        }
    }
}

/// Updates a container by stopping it, removing it and recreating it.
/// Assumes that the latest digest of the image is locally available.
async fn update_container(
    docker: &Docker,
    current_container_id: &str,
    current_container_name: &str,
    container_state: &ContainerSummaryStateEnum,
) -> Result<(), String> {
    // We start by fetching the current container configuration
    let inspect_container_options = InspectContainerOptionsBuilder::new().build();

    let container_inspect = docker
        .inspect_container(current_container_id, Some(inspect_container_options))
        .await
        .map_err(|e| {
            format!(
                "Failed to inspect container `{}`. (Internal error: `{}`)",
                current_container_name, e
            )
        })?;

    // Ensure we have a valid configuration to work with
    let new_config = container_inspect
        .config
        .ok_or_else(|| "Failed to fetch container configuration.".to_string())?;

    // Prepare the container stop, remove and create options
    let stop_container_options = StopContainerOptionsBuilder::new()
        .t(10) // Wait 1 minutes before killing the container
        .build();

    let remove_container_options = RemoveContainerOptionsBuilder::new().build();

    let create_container_options = CreateContainerOptionsBuilder::new()
        .name(container_inspect.name.as_deref().unwrap_or_default())
        .platform(container_inspect.platform.as_deref().unwrap_or_default())
        .build();

    let container_create_body: ContainerCreateBody =
        serde_json::from_value(serde_json::to_value(new_config).map_err(|e| {
            format!(
                "Failed to serialize container configuration. (Internal error: `{}`)",
                e
            )
        })?)
        .map_err(|e| {
            format!(
                "Failed to deserialize container configuration. (Internal error: `{}`)",
                e
            )
        })?;

    // Perform the container operations
    info!("\t\t-> Stopping container...");
    docker
        .stop_container(current_container_id, Some(stop_container_options))
        .await
        .map_err(|e| {
            format!(
                "Failed to stop container `{}`. (Internal error: `{}`).",
                current_container_name, e
            )
        })?;

    info!("\t\t-> Removing container...");
    docker
        .remove_container(current_container_id, Some(remove_container_options))
        .await
        .map_err(|e| {
            format!(
                "Failed to remove container `{}`. (Internal error: `{}`).",
                current_container_name, e
            )
        })?;

    info!("\t\t-> Recreating container...");
    let create_container_response = docker
        .create_container(Some(create_container_options), container_create_body)
        .await
        .map_err(|e| {
            format!(
                "Failed to create container `{}`. (Internal error: `{}`).",
                current_container_name, e
            )
        })?;

    // Restart the container if it was running before
    let was_container_running = container_state == &ContainerSummaryStateEnum::RUNNING;
    info!(
        "\t\t-> Should this container be restarted? `{}` (previous state was `{}`)",
        if was_container_running { "yes" } else { "no" },
        container_state
    );
    if container_state == &ContainerSummaryStateEnum::RUNNING {
        info!("\t\t-> Starting container...");
        let start_container_options = StartContainerOptionsBuilder::new().build();
        docker
            .start_container(
                create_container_response.id.to_owned().as_ref(),
                Some(start_container_options),
            )
            .await
            .map_err(|e| {
                format!(
                    "Failed to restart container `{}`. (Internal error: `{}`).",
                    current_container_name, e
                )
            })?;
    }

    info!("\t\t-> Container update completed successfully.");
    Ok(())
}

/// Duty: Prune `Dangling` or `All` unused images, depending on the `prune_mode` parameter.
async fn prune_images(prune_mode: &ImagesPruneMode, docker: &Docker) {
    info!("[DUTY] Pruning `{}` unused images...", prune_mode);

    let prune_images_options = PruneImagesOptionsBuilder::new()
        .filters(&HashMap::from([(
            "dangling",
            vec![matches!(prune_mode, ImagesPruneMode::Dangling).to_string()],
        )]))
        .build();

    let prune_images_response = match docker.prune_images(Some(prune_images_options)).await {
        Ok(response) => response,
        Err(e) => {
            error!("\t-> Failed to prune images. (Internal error: `{}`).", e);
            return;
        }
    };

    info!(
        "\t-> Prune completed successfully.\n\t\t- Removed `{}` images.\n\t\t- Reclaimed `{}` bytes.",
        prune_images_response
            .images_deleted
            .unwrap_or_default()
            .len(),
        prune_images_response.space_reclaimed.unwrap_or_default()
    );
}
