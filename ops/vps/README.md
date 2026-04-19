# VPS deployment (bare metal)

This project now runs with a local SQLite database file and does not require Docker.

## Runtime requirements on the VPS

- Nix with flakes enabled
- nginx
- certbot (or your existing TLS setup)
- A dedicated service user (`aenyrathia`)
- SSH key at `/home/aenyrathia/.ssh/id_ed25519` to sync wiki branches to `GIT_REMOTE`

## Files in this directory

- `aenyrathia.service`: systemd unit
- `nginx.aenyrathia.conf`: nginx vhost template
- `aenyrathia.env.example`: environment variable template
- `bootstrap.sh`: one-time host bootstrap
- `deploy.sh`: build and restart workflow

## First-time setup

1. Clone the repository to `/opt/aenyrathia/current`.
2. Run `sudo ./ops/vps/bootstrap.sh`.
3. Edit `/etc/nginx/sites-available/aenyrathia` and set the real `server_name`.
4. Edit `/etc/aenyrathia/aenyrathia.env` and set the right values.
5. Add SSH credentials for the app user:
   - `/home/aenyrathia/.ssh/id_ed25519`
   - `/home/aenyrathia/.ssh/known_hosts`
   - file mode `600`, directory mode `700`
6. Validate and reload nginx:
   - `sudo nginx -t`
   - `sudo systemctl reload nginx`
7. Issue cert:
   - `sudo certbot --nginx -d <your-domain>`
8. Build and start:
   - `sudo ./ops/vps/deploy.sh`

## Update deploy flow

1. Pull latest code in `/opt/aenyrathia/current`.
2. Run `sudo ./ops/vps/deploy.sh`.

## DNS cutover

1. Create/update A/AAAA record for your wiki domain to point to the VPS.
2. Lower TTL before cutover for faster rollback if needed.
3. After DNS propagation, verify:
   - home page load
   - login/register
   - editing/persistence
   - git sync behavior
