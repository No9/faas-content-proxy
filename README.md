# faas-content-proxy

A service to proxy a GET requests to a FaaS POST and parse the response back into HTML content.

Designed to be used with the AWS lambda container and 11ty project
https://github.com/No9/eleventy-serverless-docker

This is not meant to be a webserver in its self and should be deployed with NGINX for cacheing/tls termination etc or however you want to do your content distribution network.

## run local

Start an instance of https://github.com/No9/eleventy-serverless-docker#step-3-create-container

```
cargo run
```

Open a web browser at http://localhost:8090/1/ or http://localhost:8090/1/

## run in podpman

1. Clone this repository
    ```
    git clone https://github.com/No9/faas-content-proxy
    ```
1. Create an s3 bucket on your favourite cloud provider.

1. Copy the contents of the folder `sampledata` to the s3 bucket

1. Create some credentials for the bucket.

1. Then create a `.env` file in the base folder of this project with the following entries.

    Replacing the XXXX with the vaules from the credentials you created.

    ```
    S3ACCESSKEY=XXXX
    S3SECRET=XXXX
    S3BUCKETNAME=XXXX
    S3ENDPOINT=XXXX
    ```
1. Run podman script
    ```
    ./runpod.sh
    ```

1. Open a web browser at http://localhost:8090/1/ or http://localhost:8090/1/
