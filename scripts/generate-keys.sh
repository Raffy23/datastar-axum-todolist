ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key
openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub

ssh-keygen -t ecdsa -b 256 -m PEM -f jwtES256.key
openssl ec -in jwtES256.key -pubout -outform PEM -out jwtES256.key.pub

