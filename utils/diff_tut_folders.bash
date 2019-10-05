#!/usr/bin/env bash

DIFF=$(
    diff -uNr \
	 -x README.md \
	 -x kernel \
	 -x kernel8.img \
	 -x Cargo.lock \
	 -x target \
	 $1 $2 \
        | sed -r "s/[12][90][127][90]-.*//g"
     )

printf "\n\n## Diff to previous\n" >> "$2/README.md"
printf "\`\`\`diff\n" >> "$2/README.md"
printf "${DIFF//'diff -uNr -x README.md -x kernel -x kernel8.img -x Cargo.lock -x target'/'\ndiff -uNr'}" >> "$2/README.md"
printf "\n\`\`\`\n" >> "$2/README.md"
