# Mask 😷

Playground to learn gRPC. This repo contain 2 microservices

- hospital -> expose a list of endpoint related to the number of cases in the hospital
- pcr -> expose a list of endpoints related to the number of pcr and posiviity cases

## Internal dependency of these microservices

These two projects uses two internal library
- db -> Handle connection to the database
- utils -> Common utility method / trait... which are used by both of these microservices (maybe should have named common...)

## Install

In the hospital folder. Create a *config.toml* file which contains the credentials of the database for dev configuration. Below is an example of content

```toml
db_username="fox"
db_password="flyingfox"
db_host="localhost"
db_port="5500"
db_name="covid"
```

## Healthcheck

Healthcheck can be done with [grpc-health-probe](https://github.com/grpc-ecosystem/grpc-health-probe). Below is an exmaple of how to use it

```bash
./grpc_health_probe -addr=127.0.0.1:9000 -service=Hospital \
    -tls \
    -tls-ca-cert mask/keys/ca-cert.pem \
    -tls-server-name=localhost
```

## Development

### Docker

A docker-compose is available to run the database. It comes with the admiror interface

### Import dataset manually

You can manually import the dataset by downloading the CSV files from these links below:

- [Hospital cases by age by region](https://www.data.gouv.fr/fr/datasets/r/08c18e08-6780-452d-9b8c-ae244ad529b3)
- [Hospital new case](https://www.data.gouv.fr/fr/datasets/r/6fadff46-9efd-4c53-942a-54aca783c30c)
- [Positive pcr test by department](https://www.data.gouv.fr/fr/datasets/r/406c6a23-e283-4300-9484-54e78c8ae675)
- [Positive pcr test by region](https://www.data.gouv.fr/fr/datasets/r/001aca18-df6a-45c8-89e6-f82d689e6c01)
- [Positivity cases per department for 100k daily](https://www.data.gouv.fr/fr/datasets/r/4180a181-a648-402b-92e4-f7574647afa6)

### Import dataset automatically

A small `import.py` script can be used to import automatically the dataset into the database. The database can be configured by using the `config.toml` file. The `config.toml` file need to have this configuration:

```toml
db_username="fox"
db_password="flyingfox"
db_host="localhost"
db_port="5500"
db_name="covid"
```

### Generate cert & keys

Hospital service is configured to use TLS. These certificate & key can be generate by using the `generator.sh` bash script. Before running. You must change the subject in the `generator.sh`. Below is an example

```diff
- openssl req -x509 -newkey rsa:4096 -days 365 -keyout keys/ca-key.pem -out keys/ca-cert.pem -subj "[replace]"
+ openssl req -x509 -newkey rsa:4096 -days 365 -keyout keys/ca-key.pem -out keys/ca-cert.pem -subj "/C=FR/ST=Ile-de-france/L=Paris/O=foo/OU=bar/CN=toto/emailAddress=foo@gmail.com"

- openssl req -newkey rsa:4096 -keyout keys/server-key.pem -out keys/server-req.pem -subj "[replace]"
+ openssl req -newkey rsa:4096 -keyout keys/server-key.pem -out keys/server-req.pem -subj "/C=FR/ST=Ile-de-france/L=Lieusaint/O=foo/OU=bar/CN=tata/emailAddress=foo@gmail.com"
```

A set of keys & cert will be created in the `keys` folder. Services only uses:
- server-cert.pem
- server-key.key

### Bloom RPC TLS configuration

If you use bloom rpc for testing purposes. You can uses the same cert and key to configure TLS. Below is how you would setup the bloom rpc tls configuration

<p align="center">
  <img src="bloom.png" />
</p>

### Run the hospital service

Go to the hospital folder and run the command

```bash
cargo run
```

This will run the server port 9000. You can use tool such as [bloom RPC](https://github.com/bloomrpc/bloomrpc) and import the proto file located in the proto folder to run the service

## Tests

Each services have unit tests. These tests doesn't really test the gRPC server. But it more or less test the async function inside such as database query, simulate rpc input & output
