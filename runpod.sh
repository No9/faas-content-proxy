#! /bin/bash

podman run -dt --pod new:eleventy-fass-ondemand -p 8090:8090 quay.io/jamstacknative/faas-content-proxy
podman run -dt --pod eleventy-fass-ondemand --env-file .env quay.io/jamstacknative/eleventy-serverless-docker
