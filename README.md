# docker-tags
Print tags of docker image

## Apply to docker
This tool support docker-cli-plugin, so you can use like
```bash
docker tags alpine
```
If you want to use below, install this plugin into
  * `/usr/local/lib/docker/cli-plugins/` or 
  * `$HOME/.docker/cli-plugins/`.
```bash
# For local user
mkdir -p $HOME/.docker/cli-plugins
wget -P $HOME/.docker/cli-plugins/ http://github.com/miyake13000/docker-tags/releases/latest/download/docker-tags

# For all user
sudo mkdir -p /usr/local/lib/docker/cli-plugins
wget -P /usr/local/lib/docker/cli-plugins http://github.com/miyake13000/docker-tags/releases/latest/download/docker-tags
```

## Usage
```bash
$ docker tags alpine
latest
3.19.1
3.19
3.18.6
:
```
* with last update time of image
  ```bash
  $ docker tags -u alpine
  latest (2024-01-27)
  3.19.1 (2024-01-27)
  3.19 (2024-01-27)
  3.18.6 (2024-01-27)
  :
  ```

## Build
```bash
git clone https://github.com/miyake13000/docker-tags && cd docker-tags
cargo build --release
```
