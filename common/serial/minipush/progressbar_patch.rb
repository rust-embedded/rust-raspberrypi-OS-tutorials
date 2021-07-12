# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

# Monkey-patch ruby-progressbar so that it supports reporting the progress in KiB instead of Byte.

class ProgressBar
    # Add kibi version of progress
    class Progress
        def progress_kibi
            @progress / 1024
        end
    end

    module Format
        # Add new formatting option
        class Molecule
            MOLECULES_EXTENDED = MOLECULES.dup
            MOLECULES_EXTENDED[:k] = %i[progressable progress_kibi]

            def initialize(letter)
                self.key         = letter
                self.method_name = MOLECULES_EXTENDED.fetch(key.to_sym)
            end
        end
    end
end
