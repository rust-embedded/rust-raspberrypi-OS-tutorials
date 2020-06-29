#!/usr/bin/env bash

git config core.hooksPath .githooks
bundle config set path '.vendor/bundle'
bundle install
