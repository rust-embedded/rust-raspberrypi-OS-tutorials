#!/usr/bin/env bash
#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

tmux new-session -d -s raspi3 &
sleep 1
tmux new-window -t raspi3:1
tmux new-window -t raspi3:2

tmux send-keys -t raspi3:0 "clear; echo '=== MiniUart ==='; bash /dev/ptmx" C-m
tmux send-keys -t raspi3:1 "clear; printf '=== PL011 Uart ===\n\n';bash /dev/ptmx" C-m

FIRST=$(ps aux | grep ptmx | sort | awk '{print $7}' | sed '1q;d')
SECOND=$(ps aux | grep ptmx | sort | awk '{print $7}' | sed '2q;d')

tmux send-keys -t raspi3:2 "clear; cat /emulation/instructions.txt && qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial /dev/$SECOND -serial /dev/$FIRST && tmux kill-session" C-m

tmux join-pane -s raspi3:0 -t 2
tmux join-pane -s raspi3:1 -t 2

tmux select-pane -t 1
tmux attach-session -t raspi3
