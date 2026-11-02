# docker-tags
Print tags of docker image

## Install (apply to docker)
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
chmod +x $HOME/.docker/cli-plugins/docker-tags

# For all user
sudo mkdir -p /usr/local/lib/docker/cli-plugins
sudo wget -P /usr/local/lib/docker/cli-plugins http://github.com/miyake13000/docker-tags/releases/latest/download/docker-tags
sudo chmod +x /usr/local/lib/docker/cli-plugins/docker-tags
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

### Compare tags by digest

- `--show-tag=TAG` annotates tags sharing TAG's digest with `(TAG)`.
- `--show-latest` / `-s` is shorthand for `--show-tag=latest`.
- `--filter-tag=TAG` lists tags sharing TAG's digest, excluding TAG itself.
- `--filter-latest` / `-l` is shorthand for `--filter-tag=latest`.

```bash
$ docker tags -s ubuntu
26.10
26.04 (latest)
...
$ docker tags -l ubuntu
26.04
resolute
$ docker tags --filter-tag=26.04 ubuntu
latest (26.04)
26.10
...
$ docker tags --show-tag=26.04 -u ubuntu
latest
resolute
```


## Build
```bash
git clone https://github.com/miyake13000/docker-tags && cd docker-tags
cargo build --release
```
