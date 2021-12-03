# faas-content-proxy

A service to proxy a GET requests to a FaaS POST event and parse the response back into HTML content.

Designed to be used with the AWS lambda container and 11ty project.

This is currently not meant to be an internet facing webserver and should be deployed with a caching layer and TLS termination etc.

## usage

### Step 1. Build the project
```
cargo build --release
```
### Step 2. Edit the knative.toml 
If you are just running the demo project https://github.com/No9/eleventy-serverless-docker then the toml file will be fine.
Otherwise review the following settings in `knative.toml`

```
[build]
# if you want to publish content you can by pointing to a folder.
publish = "public"

[errorpages]
# Currently only custom 404 and 500 are supported.
not_found = "404.html"
internal_server_error = "500.html"

[[headers]]
  # Define which paths this specific [[headers]] block will cover.
  for = "/*"

  [headers.values]
    X-Frame-Options = "DENY"
    X-XSS-Protection = "1; mode=block"
    Content-Security-Policy = "frame-ancestors https://www.facebook.com"

    # Multi-value headers are expressed with multi-line strings.
	cache-control = '''
	max-age=0,
	no-cache,
	no-store,
	must-revalidate'''

[[redirects]]
# this section matches routes to a serverless function.
from = "/:slug/"
to = "http://127.0.0.1:8080/2015-03-31/functions/function/invocations"
```



## Step 3. Run an 11ty serverless instance.

If your just taking a look then start an instance of https://github.com/No9/eleventy-serverless-docker#step-3-create-container


## Step 4. View the dynamic content.

Open a web browser at http://localhost:8090/1/ or http://localhost:8090/1/



## run the whole project in podpman

If you have podman installed then the whole demo can be ran as follows:

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
