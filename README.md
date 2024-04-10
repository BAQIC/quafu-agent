# quafu-agent
## how to use
```bash
docker pull ghcr.io/baqic/quafu-agent:main
docker run -d --network=host --name=quafu-agent --restart=always ghcr.io/baqic/quafu-agent:main
```

## check the logs
```bash
# if the name of your container is quafu-agent
docker exec -it quafu-agent cat /home/sq/quafu-agent/log/requests.log
```

## check container logs
```bash
docker logs quafu-agent
```