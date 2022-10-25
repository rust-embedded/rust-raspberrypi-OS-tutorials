#!/usr/bin/env bash

git config core.hooksPath .githooks

#
# Ruby and Bundler
#
if ! command -v bundle &> /dev/null
then
    echo "'bundle' could not be found. Please install Ruby and Bundler."
    exit
fi
bundle config set --local path '.vendor/bundle'
bundle install

#
# NPM
#
if ! command -v npm &> /dev/null
then
    echo "'npm' could not be found. Please install it."
    exit
fi
npm install --save-dev --save-exact prettier

#
# Misspell
#
if ! command -v curl &> /dev/null
then
    echo "'curl' could not be found. Please install it."
    exit
fi
curl -L -o ./install-misspell.sh https://raw.githubusercontent.com/client9/misspell/master/install-misspell.sh
sh ./install-misspell.sh -b .vendor
rm install-misspell.sh
