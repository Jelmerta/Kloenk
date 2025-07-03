#!/bin/sh

# Currently these steps are executed manually because some interaction is required, next time we setup a server maybe try to make this script executable in one go

sudo apt update

# First: Install Docker! https://docs.docker.com/engine/install/debian/#install-using-the-repository
# Manual script to run on the server to initialise a server such that we can externally give a sign to retrieve and deploy the container with the game.
# This only needs to be run once for each server.
# Based on: https://nbailey.ca/post/github-actions-ssh/

sudo useradd --create-home --user-group --shell /bin/bash --groups docker deploy
sudo usermod --lock deploy
sudo -i -u deploy
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -C "deploy@server"
touch authorized_keys
cat .ssh/id_ed25519.pub > .ssh/authorized_keys
# cat .ssh/id_ed25519 and save the string to clipboard for next step
# cat .ssh/id_ed25519.pub and save the string to clipboard for next step

# Perform the necessary GitHub steps, add 3 secrets and a deploy key

# Make sure firewall allows access on 80/443 for http/https and 22 for ssh
apt-get install ufw
ufw enable
ufw allow ssh
ufw allow http
ufw allow https

# Connecting to the server can be done as such after setting up ssh:
#ssh -i ~/.ssh/id_ed25519_kloenk root@xxx.xxx.xxx.xxx

# Setup certbot to get certs from lets encrypt for SSL/HTTPS:
# https://certbot.eff.org/instructions?ws=nginx&os=pip
sudo apt install python3 python3-dev python3-venv libaugeas-dev gcc
sudo python3 -m venv /opt/certbot/
sudo /opt/certbot/bin/pip install --upgrade pip
sudo /opt/certbot/bin/pip install certbot
sudo ln -s /opt/certbot/bin/certbot /usr/bin/certbot

# setup acme challenges to refresh certs
sudo mkdir -p /var/www/certbot-acme
sudo chmod -R 755 /var/www/certbot-acme

# First time we need to setup ssl as such:
sudo certbot certonly --webroot \
  -w /var/www/certbot-acme \
  --domain hatsu.tech \
  --email myemailaddress@hatsu.tech \
  --non-interactive \
  --agree-tos

# To set up a certbot, we create a new directory to add the certs to, as certbot keeps symbolic links to a different location and we want the full file to be mounted as a volume when starting the docker container
sudo mkdir -p /etc/ssl-kloenk
sudo cp -L /etc/letsencrypt/live/hatsu.tech/cert.pem /etc/ssl-kloenk/cert.pem
sudo cp -L /etc/letsencrypt/live/hatsu.tech/privkey.pem /etc/ssl-kloenk/privkey.pem
sudo cp -L /etc/letsencrypt/live/hatsu.tech/chain.pem /etc/ssl-kloenk/chain.pem
sudo cp -L /etc/letsencrypt/live/hatsu.tech/fullchain.pem /etc/ssl-kloenk/fullchain.pem

# To renew the certificates automatically, we setup a cron job
echo "0 0,12 * * * root /opt/certbot/bin/python -c 'import random; import time; time.sleep(random.random() * 3600)' && sudo certbot renew -q --webroot -w /var/www/certbot-acme" | sudo tee -a /etc/crontab > /dev/null

# Shared volume where static assets are built and served from
docker volume create kloenk-static