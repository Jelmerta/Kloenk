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