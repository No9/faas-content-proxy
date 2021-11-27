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

