#!/bin/sh

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

# Make sure firewall allows access on 80/443 for http/https
apt-get install ufw
ufw enable
ufw allow ssh
ufw allow http
ufw allow https

# Move the SSL certificate and private key over (either with scp or copying the contents)
# Or setup certbot to get certs from lets encrypt:
# https://certbot.eff.org/instructions?ws=nginx&os=pip
# To set up a certbot, we create a new directory to add the certs to, as certbot keeps symbolic links to a different location and we want the full file to be mounted as a volume when starting the docker container
sudo mkdir -p /etc/ssl-kloenk
sudo cp -L /etc/letsencrypt/live/hatsu.tech/cert.pem /etc/ssl-kloenk/cert.pem
sudo cp -L /etc/letsencrypt/live/hatsu.tech/privkey.pem /etc/ssl-kloenk/privkey.pem

# Connecting to the server can be done as such:
ssh -i ~/.ssh/id_ed25519_kloenk root@xxx.xxx.xxx.xxx