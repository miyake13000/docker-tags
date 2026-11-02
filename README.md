# docker-tags
Print all tags of specified image

## Install
```bash
mkdir -p "$HOME/.docker/cli-plugins"
curl -fsSL "https://github.com/miyake13000/docker-tags/releases/latest/download/docker-tags-linux-$(uname -m).gz" | gzip -d > "$HOME/.docker/cli-plugins/docker-tags"
chmod +x "$HOME/.docker/cli-plugins/docker-tags"

# Or, install for all users
sudo mkdir -p "/usr/local/lib/docker/cli-plugins"
sudo bash -c 'curl -fsSL "https://github.com/miyake13000/docker-tags/releases/latest/download/docker-tags-linux-$(uname -m).gz" | gzip -d > "/usr/local/lib/docker/cli-plugins"'
sudo chmod +x "/usr/local/lib/docker/cli-plugins/docker-tags"
```

## How to use
```bash
$ docker tags alpine
latest
3.19.1
3.19
3.18.6
:

# With last update time
$ docker tags -u alpine
latest (2024-01-27)
3.19.1 (2024-01-27)
3.19 (2024-01-27)
3.18.6 (2024-01-27)
:
```

### Filter / Annotate with specified tags

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
