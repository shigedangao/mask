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

## Run the project

The hospital contains gRPC service which allows to retrieve the hospitalization by age and region. Check the `hospitalization.proto` file for the definition.
