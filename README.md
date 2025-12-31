# Docker-maid [![Publish Docker image](https://github.com/WarPlexTech/docker-maid/actions/workflows/publish_image.yml/badge.svg)](https://github.com/WarPlexTech/docker-maid/actions/workflows/publish_image.yml)

A simple and lightweight Docker housekeeping tool that helps you keep your system clean and your containers up to date.

## Features

- **Container Updates:** Automatically update containers when a new version of their image is available.
- **Image Pruning:** Remove unused or dangling images to save disk space.
- **Build Cache Pruning:** Keep your Docker build cache under control.
- **Scheduled Rounds:** Run housekeeping on a customizable cron schedule.

## Quick-start

The easiest way to run `docker-maid` is using Docker Compose.

```yaml
services:
  docker-maid:
    image: ghcr.io/warplextech/docker-maid:latest
    container_name: docker-maid
    tty: true # Optional: enables colorized logs
    restart: always
    environment:
      # Schedule in cron format (defaults to every 6 hours).
      # Note: This uses an expanded cron format:
      # `sec` `min` `hour` `day of month` `month` `day of week` `year`
      - MAID_SCHEDULE=0 0 */6 * * * *

      # Set to `true` to run an housekeeping round on container startup
      - MAID_RUN_ON_STARTUP=false

      # How to handle containers when newer image digests are available. Options:
      # - label: [Not yet implemented] Will allow choosing a strategy via container labels.
      # - update: Recreate containers using the latest image digest.
      # - notify: [Not yet implemented] Currently acts as a `dry-run` option.
      # - none (default): Do not check for image updates.
      - MAID_DUTY_CONTAINERS_UPDATES=update

      # How to handle images that are not used by any container. Options are:
      # - all: Prune all unused images
      # - dangling: Prune only dangling images
      # - none (default): Do not prune images
      - MAID_DUTY_PRUNE_IMAGES=all

      # How to handle Docker build cache. Options are:
      # - all: Prune all Docker build cache
      # - none (default): Do not prune Docker build cache
      - MAID_DUTY_PRUNE_BUILD_CACHE=all
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
```

## Configuration

| Environment Variable           | Options                            | Default           | Description                                                                             |
|--------------------------------|------------------------------------|-------------------|-----------------------------------------------------------------------------------------|
| `MAID_SCHEDULE`                | Cron expression                    | `0 0 */6 * * * *` | Housekeeping schedule (`sec` `min` `hour` `day of month` `month` `day of week` `year`). |
| `MAID_RUN_ON_STARTUP`          | `true`,<br/>`false`                | `false`           | Run a housekeeping round immediately when the container starts.                         |
| `MAID_DUTY_CONTAINERS_UPDATES` | `update`,<br/>`notify`,<br/>`none` | `none`            | `update` recreates containers. `notify` currently acts as a dry-run.                    |
| `MAID_DUTY_PRUNE_IMAGES`       | `all`,<br/>`dangling`,<br/>`none`  | `none`            | Prune unused or just dangling images.                                                   |
| `MAID_DUTY_PRUNE_BUILD_CACHE`  | `all`,<br/>`none`                  | `none`            | Prune all Docker build cache.                                                           |

## Contributing

This is my first Rust project, and I'll be improving it as I continue to learn. Contributions, suggestions, and feedback are welcome 😄

