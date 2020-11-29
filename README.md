# Rabbit MQ Rust

rust final exresice.

## docker
docker run -d --hostname my-rabbit --name some-rabbit -p 8080:15672 -p 5672:5672 rabbitmq:3-management  
this command run docker of Rabbit mq server that listen to localhost:5672 and exports management to port 8080.