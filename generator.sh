rm keys/*.pem
rm keys/*.srl

# based on @link: https://dev.to/techschoolguru/how-to-create-sign-ssl-tls-certificates-2aai
# generate self signing certificate for Certificate authority (CA)
openssl req -x509 -newkey rsa:4096 -days 365 -keyout keys/ca-key.pem -out keys/ca-cert.pem -subj "[replace]"

# generate server private key and csr
# we don't want to self-sign it. We're going to sign these key with the self sign key that we generated with the previous command
openssl req -newkey rsa:4096 -keyout keys/server-key.pem -out keys/server-req.pem -subj "[replace]"

# sign the server key with the CA key
openssl x509 -req -in keys/server-req.pem -days 60 -CA keys/ca-cert.pem -CAkey keys/ca-key.pem -CAcreateserial -out keys/server-cert.pem -extfile keys/server-ext.cnf


# verify that the cert is valid
# openssl verify -CAfile keys/ca-cert.pem keys/server-cert.pem

# output the private key from the server-key.pem
openssl pkey -in keys/server-key.pem -out keys/server-key.key
