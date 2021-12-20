# Mask 😷

Playground to learn gRPC

## Install

In the hospital folder. Create a *config.toml* file which contains the credentials of the database for dev configuration. Below is an example of content

```toml
db_username="fox"
db_password="flyingfox"
db_host="localhost"
db_port="5500"
db_name="covid"
```

## Development

### Docker

A docker-compose is available to run the database. It comes with the admiror interface

### Import dataset

First you'll need to download the necessary data.

- [Hospital cases by age](https://www.data.gouv.fr/fr/datasets/r/08c18e08-6780-452d-9b8c-ae244ad529b3)
- [Hospital new case](https://www.data.gouv.fr/fr/datasets/r/6fadff46-9efd-4c53-942a-54aca783c30c)

You can feed the database by running the `import.py` file

### Run the hospital service

Go to the hospital folder and run the command

```bash
cargo run
```

This will run the server port 9000. You can use tool such as [bloom RPC](https://github.com/bloomrpc/bloomrpc) and import the proto file located in the proto folder to run the service
