#!/usr/bin/env bash

# SPDX-License-Identifier: MIT
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>

DIFF=$(
    diff -uNr \
	 -x README.md \
	 -x kernel \
	 -x kernel8.img \
	 -x Cargo.lock \
	 -x target \
	 $1 $2 \
	| sed -r "s/[12][90][127][90]-.*//g" \
	| sed -r "s/[[:space:]]*$//g" \
	| sed -r "s/%/modulo/g" \
        | sed -r "s/diff -uNr -x README.md -x kernel -x kernel8.img -x Cargo.lock -x target/\ndiff -uNr/g"
     )

HEADER="## Diff to previous"
ORIGINAL=$(
    cat $2/README.md \
	| sed -rn "/$HEADER/q;p"
	)

echo "$ORIGINAL" > "$2/README.md"
printf "\n$HEADER\n" >> "$2/README.md"
printf "\`\`\`diff\n" >> "$2/README.md"
echo "$DIFF" >> "$2/README.md"
printf "\n\`\`\`\n" >> "$2/README.md"
